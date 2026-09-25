#![deny(clippy::all)]

pub use self::{
    jsx::{Options, UselessFlags, jsx},
    pure_annotations::pure_annotations,
    refresh::{options::RefreshOptions, refresh},
};
use swc_core::{
    common::{
        Mark, SourceMapper, comments::Comments,
        plugin::metadata::TransformPluginMetadataContextKind, sync::Lrc,
    },
    ecma::ast::{Pass, Program},
    plugin::{errors::HANDLER, plugin_transform, proxies::TransformPluginProgramMetadata},
};

mod inferno_flags;
mod jsx;
mod program_bindings;
mod pure_annotations;
mod refresh;
mod transformations;
mod warnings;

/// Runs the fast refresh pass (when enabled), the pure annotation pass and the JSX transform.
///
/// The program must have been processed by swc's `resolver`; `unresolved_mark` is the unresolved
/// [Mark] passed to it. `cm` is only read by the fast refresh pass: pass the program's
/// `SourceMap`, or the plugin metadata's source map proxy.
///
/// # Note
///
/// Errors and warnings are reported through `swc_core::common::errors::HANDLER`.
pub fn inferno<C, S>(
    cm: Lrc<S>,
    comments: Option<C>,
    mut options: Options,
    unresolved_mark: Mark,
) -> impl Pass
where
    C: Comments + Clone,
    S: SourceMapper,
{
    let development = options.development();
    let refresh_pass = options
        .refresh
        .take()
        .filter(|_| development)
        .map(|refresh_options| refresh(refresh_options, cm, comments.clone()));
    let pure_pass = options
        .pure()
        .then(|| pure_annotations(comments.clone(), options.import_source().into()));

    // The pure annotation pass runs first: the JSX transform annotates the calls it generates,
    // and a file without hand-written Inferno imports is not walked again
    (
        refresh_pass,
        pure_pass,
        jsx(comments, options, unresolved_mark),
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

    let filename = metadata
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .unwrap_or_else(|| "unknown file".into());
    let cm = Lrc::new(metadata.source_map);
    let (program, warnings) = warnings::collect_warnings(&*cm, &filename, || {
        program.apply(inferno(
            cm.clone(),
            metadata.comments,
            options,
            metadata.unresolved_mark,
        ))
    });

    // swc drops the warnings of a transform that succeeds, so they are printed like
    // babel-plugin-inferno prints them with console.warn
    for warning in warnings {
        eprintln!("{warning}");
    }

    program
}
