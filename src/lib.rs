#![deny(clippy::all)]
#![allow(clippy::arc_with_non_send_sync)]

pub use self::{
    jsx::*,
    pure_annotations::pure_annotations,
    refresh::{options::RefreshOptions, refresh},
};
use swc_core::ecma::ast::Pass;
use swc_core::{
    common::{Mark, SourceMap, comments::Comments, sync::Lrc},
    ecma::ast::Program,
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

mod inferno_flags;
mod jsx;
mod pure_annotations;
mod refresh;
mod transformations;

///
/// `top_level_mark` should be [Mark] passed to
/// [swc_ecma_transforms_base::resolver::resolver_with_mark].
///
///
///
/// # Note
///
/// This pass uses [swc_ecma_utils::HANDLER].
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
    let Options { development, .. } = options;
    let development = development.unwrap_or(false);

    let refresh_options = options.refresh.take();

    (
        refresh(
            development,
            refresh_options,
            cm.clone(),
            comments.clone(),
            top_level_mark,
        ),
        jsx(comments.clone(), options, unresolved_mark),
        pure_annotations(comments),
    )
}

#[plugin_transform]
fn inferno_jsx_plugin(mut program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let top_level_mark = Mark::new();
    let cm = Lrc::new(SourceMap::default());
    let unresolved_mark = metadata.unresolved_mark;

    let options: Options = Default::default();
    let development = options.development.unwrap_or(false);

    if development {
        let refresh_options = options.clone().refresh;
        let mut refresh_pass = refresh(
            development,
            refresh_options,
            cm.clone(),
            Some(&metadata.comments),
            top_level_mark,
        );
        program = program.apply(&mut refresh_pass);
    }

    let mut jsx_pass = jsx(Some(&metadata.comments), options, unresolved_mark);
    program = program.apply(&mut jsx_pass);

    let mut pure_pass = pure_annotations(Some(&metadata.comments));
    program.apply(&mut pure_pass)
}
