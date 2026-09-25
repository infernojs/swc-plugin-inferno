#![deny(clippy::all)]

pub use self::{
    jsx::{Options, jsx},
    pure_annotations::pure_annotations,
    refresh::{options::RefreshOptions, refresh},
};
use swc_core::{
    common::{Mark, SourceMap, comments::Comments, sync::Lrc},
    ecma::ast::{Pass, Program},
    plugin::{errors::HANDLER, plugin_transform, proxies::TransformPluginProgramMetadata},
};

mod inferno_flags;
mod jsx;
mod pure_annotations;
mod refresh;
mod transformations;

/// Runs the fast refresh pass (when enabled), the JSX transform and the pure annotation pass.
///
/// `top_level_mark` and `unresolved_mark` should be the marks passed to swc's `resolver`.
///
/// # Note
///
/// Errors are reported through `swc_core::common::errors::HANDLER`.
pub fn inferno<C>(
    cm: Lrc<SourceMap>,
    comments: Option<C>,
    mut options: Options,
    top_level_mark: Mark,
    unresolved_mark: Mark,
) -> impl Pass
where
    C: Comments + Clone,
{
    let development = options.development();
    let refresh_pass = options
        .refresh
        .take()
        .filter(|_| development)
        .map(|refresh_options| {
            refresh(
                true,
                Some(refresh_options),
                cm,
                comments.clone(),
                top_level_mark,
            )
        });
    let pure_pass = options.pure().then(|| pure_annotations(comments.clone()));

    (
        refresh_pass,
        jsx(comments, options, unresolved_mark),
        pure_pass,
    )
}

#[plugin_transform]
fn inferno_jsx_plugin(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let options = match metadata
        .get_transform_plugin_config()
        .map(|config| serde_json::from_str::<Option<Options>>(&config))
    {
        None => Options::default(),
        Some(Ok(options)) => options.unwrap_or_default(),
        Some(Err(err)) => {
            HANDLER.with(|handler| {
                handler.err(&format!(
                    "swc-plugin-inferno: invalid plugin options: {err}"
                ))
            });
            return program;
        }
    };

    program.apply(inferno(
        Lrc::new(SourceMap::default()),
        metadata.comments,
        options,
        Mark::new(),
        metadata.unresolved_mark,
    ))
}
