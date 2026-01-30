#![allow(clippy::redundant_allocation)]

use crate::transformations::lowercase_attrs::requires_lowercasing;
use crate::transformations::parse_vnode_flag::parse_vnode_flag;
use crate::transformations::transform_attribute::transform_attribute;
use crate::VNodeType::Component;
use crate::{
    inferno_flags::{ChildFlags, VNodeFlags},
    refresh::options::{deserialize_refresh, RefreshOptions},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use swc_atoms::Wtf8Atom;
use swc_config::merge::Merge;
use swc_core::atoms::atom;
use swc_core::common::comments::Comments;
use swc_core::common::util::take::Take;
use swc_core::common::{FileName, Mark, SourceMap, Span, Spanned, SyntaxContext, DUMMY_SP};
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::Atom;
use swc_core::ecma::utils::{
    drop_span, prepend_stmt, quote_ident, swc_atoms, ExprFactory, StmtLike,
};
use swc_core::ecma::visit::{noop_visit_mut_type, visit_mut_pass, VisitMut, VisitMutWith};
use swc_core::plugin::errors::HANDLER;
use swc_ecma_parser::{parse_file_as_expr, Syntax};

#[cfg(test)]
mod tests;

mod attr;
mod text;
mod vnode_args;

use self::attr::{jsx_attr_value_to_expr, jsx_attr_value_to_expr_or_invalid};
use self::text::jsx_text_to_str;
use self::vnode_args::{create_component_vnode_args, create_fragment_vnode_args, CreateVNodeArgs};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq, Merge)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Options {
    /// If this is `true`, swc will behave just like babel 8 with
    /// `BABEL_8_BREAKING: true`.
    #[serde(skip, default)]
    pub next: Option<bool>,

    #[serde(default)]
    pub import_source: Option<String>,

    #[serde(default)]
    pub development: Option<bool>,

    #[serde(default, deserialize_with = "deserialize_refresh")]
    // default to disabled since this is still considered as experimental by now
    pub refresh: Option<RefreshOptions>,
}

pub fn default_import_source() -> String {
    "inferno".into()
}

pub fn parse_expr_for_jsx(
    cm: &SourceMap,
    name: &str,
    src: String,
    top_level_mark: Mark,
) -> Arc<Box<Expr>> {
    let fm = cm.new_source_file(
        FileName::Custom(format!("<jsx-config-{name}.js>")).into(),
        src,
    );

    parse_file_as_expr(
        &fm,
        Syntax::default(),
        Default::default(),
        None,
        &mut vec![],
    )
    .map_err(|e| {
        HANDLER.with(|h| {
            e.into_diagnostic(h)
                .note("error detected while parsing option for classic jsx transform")
                .emit()
        })
    })
    .map(drop_span)
    .map(|mut expr| {
        apply_mark(&mut expr, top_level_mark);
        expr
    })
    .map(Arc::new)
    .unwrap_or_else(|()| Arc::new(Box::new(Expr::Invalid(Invalid { span: DUMMY_SP }))))
}

fn apply_mark(e: &mut Expr, mark: Mark) {
    match e {
        Expr::Ident(i) => {
            i.ctxt = i.ctxt.apply_mark(mark);
        }
        Expr::Member(MemberExpr { obj, .. }) => {
            apply_mark(obj, mark);
        }
        _ => {}
    }
}

fn named_import_exists(import_name: &Ident, import: &ImportDecl) -> bool {
    import.specifiers.iter().any(|specifier| {
        matches!(
            specifier,
            ImportSpecifier::Named(named) if import_name.sym == named.local.sym
        )
    })
}

fn merge_imports(
    imports: &[Ident],
    default_import_src: &Wtf8Atom,
    stmts: &mut Vec<ModuleItem>,
) -> bool {
    for stmt in stmts {
        if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = stmt {
            if import.src.value == *default_import_src {
                for specifier in &import.specifiers {
                    if let ImportSpecifier::Namespace(_) = specifier {
                        // Do not try to merge with * As FooBar import statements
                        return false;
                    }
                }

                for import_to_add in imports {
                    let import_exists = named_import_exists(import_to_add, import);

                    if !import_exists {
                        import
                            .specifiers
                            .push(ImportSpecifier::Named(ImportNamedSpecifier {
                                span: DUMMY_SP,
                                local: import_to_add.clone(),
                                imported: None,
                                is_type_only: false,
                            }))
                    }
                }

                return true;
            }
        }
    }

    false
}

#[derive(PartialEq)]
pub enum VNodeType {
    Element = 0,
    Component = 1,
    Fragment = 2,
}

///
/// Turn JSX into Inferno function calls
///
///
/// `top_level_mark` should be [Mark] passed to
/// [swc_ecma_transforms_base::resolver::resolver_with_mark].
pub fn jsx<C>(comments: Option<C>, options: Options, unresolved_mark: Mark) -> impl Pass
where
    C: Comments,
{
    visit_mut_pass(Jsx {
        unresolved_mark,
        import_source: options
            .import_source
            .unwrap_or_else(default_import_source)
            .into(),
        import_create_vnode: None,
        import_create_component: None,
        import_create_text_vnode: None,
        import_create_fragment: None,
        import_normalize_props: None,

        comments,
        top_level_node: true,
    })
}

struct Jsx<C>
where
    C: Comments,
{
    unresolved_mark: Mark,

    import_source: Wtf8Atom,

    import_create_vnode: Option<Ident>,
    import_create_component: Option<Ident>,
    import_create_text_vnode: Option<Ident>,
    import_create_fragment: Option<Ident>,
    import_normalize_props: Option<Ident>,
    top_level_node: bool,

    comments: Option<C>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct JsxDirectives {
    pub import_source: Option<Atom>,
}

impl<C> Jsx<C>
where
    C: Comments,
{
    fn inject_runtime<T, F>(&mut self, body: &mut Vec<T>, inject: F)
    where
        T: StmtLike,
        F: Fn(Vec<Ident>, Wtf8Atom, &mut Vec<T>),
    {
        let mut import_specifiers: Vec<Ident> = Vec::with_capacity(5);

        if let Some(_local) = self.import_create_vnode.take() {
            import_specifiers.push(quote_ident!("createVNode").into())
        }
        if let Some(_local) = self.import_create_component.take() {
            import_specifiers.push(quote_ident!("createComponentVNode").into())
        }
        if let Some(_local) = self.import_create_text_vnode.take() {
            import_specifiers.push(quote_ident!("createTextVNode").into())
        }
        if let Some(_local) = self.import_normalize_props.take() {
            import_specifiers.push(quote_ident!("normalizeProps").into())
        }
        if let Some(_local) = self.import_create_fragment.take() {
            import_specifiers.push(quote_ident!("createFragment").into())
        }

        if !import_specifiers.is_empty() {
            inject(import_specifiers, self.import_source.clone(), body);
        }
    }

    fn set_local_import_refs(&mut self, stmts: &mut Vec<ModuleItem>) {
        for stmt in stmts {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = stmt {
                if import.src.value == self.import_source {
                    for specifier in import.specifiers.iter_mut() {
                        match specifier {
                            ImportSpecifier::Named(named_import) => {
                                if named_import.local.sym == "createVNode" {
                                    self.import_create_vnode
                                        .get_or_insert(named_import.local.clone());
                                } else if named_import.local.sym == "createComponentVNode" {
                                    self.import_create_component
                                        .get_or_insert(named_import.local.clone());
                                } else if named_import.local.sym == "createTextVNode" {
                                    self.import_create_text_vnode
                                        .get_or_insert(named_import.local.clone());
                                } else if named_import.local.sym == "createFragment" {
                                    self.import_create_fragment
                                        .get_or_insert(named_import.local.clone());
                                } else if named_import.local.sym == "normalizeProps" {
                                    self.import_normalize_props
                                        .get_or_insert(named_import.local.clone());
                                }
                            }
                            _ => continue,
                        }
                    }

                    return;
                }
            }
        }
    }

    fn jsx_frag_to_expr(&mut self, el: JSXFragment) -> Expr {
        let span = el.span();

        if let Some(comments) = &self.comments {
            comments.add_pure_comment(span.lo);
        }

        let fragment = self
            .import_create_fragment
            .get_or_insert_with(|| quote_ident!("createFragment").into())
            .clone();

        let mut children_requires_normalization: bool = false;
        let mut parent_can_be_keyed: bool = false;
        let mut children_count: u16 = 0;

        let mut children = vec![];
        for child in el.children {
            let child_expr = Some(match child {
                JSXElementChild::JSXText(text) => {
                    // TODO(kdy1): Optimize
                    let value: swc_atoms::Wtf8Atom = jsx_text_to_str(&*text.value);
                    let s = Str {
                        span: text.span,
                        raw: None,
                        value,
                    };

                    if s.value.is_empty() {
                        continue;
                    }

                    ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Call(CallExpr {
                            span: DUMMY_SP,
                            callee: self
                                .import_create_text_vnode
                                .get_or_insert_with(|| quote_ident!("createTextVNode").into())
                                .clone()
                                .as_callee(),
                            args: vec![s.as_arg()],
                            ..Default::default()
                        })),
                    }
                }
                JSXElementChild::JSXExprContainer(JSXExprContainer {
                    expr: JSXExpr::Expr(e),
                    ..
                }) => {
                    children_requires_normalization = true;
                    parent_can_be_keyed = false;
                    e.as_arg()
                }
                JSXElementChild::JSXExprContainer(JSXExprContainer {
                    expr: JSXExpr::JSXEmptyExpr(..),
                    ..
                }) => continue,
                JSXElementChild::JSXElement(el) => {
                    if !parent_can_be_keyed && !children_requires_normalization {
                        // Loop direct children to check if they have key property set
                        parent_can_be_keyed = Self::does_children_have_key_defined(&el);
                    }
                    self.jsx_elem_to_expr(*el).as_arg()
                }
                JSXElementChild::JSXFragment(el) => self.jsx_frag_to_expr(el).as_arg(),
                JSXElementChild::JSXSpreadChild(JSXSpreadChild { span, expr, .. }) => {
                    ExprOrSpread {
                        spread: Some(span),
                        expr,
                    }
                }
            });

            children_count += 1;

            children.push(child_expr)
        }

        let child_flags;

        if !children_requires_normalization {
            if children_count >= 1 {
                if parent_can_be_keyed {
                    child_flags = ChildFlags::HasKeyedChildren;
                } else {
                    child_flags = ChildFlags::HasNonKeyedChildren;
                }
            } else {
                child_flags = ChildFlags::HasInvalidChildren;
            }
        } else {
            child_flags = ChildFlags::UnknownChildren;
        };

        Expr::Call(CallExpr {
            span,
            callee: fragment.as_callee(),
            args: create_fragment_vnode_args(children, false, child_flags as u16, None, None),
            type_args: None,
            ..Default::default()
        })
    }

    fn jsx_elem_to_expr(&mut self, el: JSXElement) -> Expr {
        let top_level_node = self.top_level_node;
        let span = el.span();
        self.top_level_node = false;
        let unresolved_ctxt = SyntaxContext::empty().apply_mark(self.unresolved_mark);

        if let Some(comments) = &self.comments {
            comments.add_pure_comment(span.lo);
        }

        let name_span: Span = el.opening.name.span();
        let name_expr;
        let mut mut_flags: u16;
        let vnode_kind: VNodeType;

        match el.opening.name {
            JSXElementName::Ident(ident) => {
                if ident.sym == "this" {
                    vnode_kind = Component;
                    mut_flags = VNodeFlags::ComponentUnknown as u16;
                    name_expr = Expr::This(ThisExpr { span: name_span });
                } else if is_component_vnode(&ident) {
                    if ident.sym == "Fragment" {
                        vnode_kind = VNodeType::Fragment;
                        mut_flags = VNodeFlags::ComponentUnknown as u16;
                        name_expr = Expr::Ident(Ident::new(
                            "createFragment".into(),
                            ident.span,
                            Default::default(),
                        ));
                    } else {
                        vnode_kind = Component;
                        mut_flags = VNodeFlags::ComponentUnknown as u16;
                        name_expr = Expr::Ident(ident)
                    }
                } else {
                    vnode_kind = VNodeType::Element;
                    mut_flags = parse_vnode_flag(&ident.sym);
                    name_expr = Expr::Lit(Lit::Str(Str {
                        span: name_span,
                        raw: None,
                        value: ident.sym.into(),
                    }))
                }
            }
            JSXElementName::JSXNamespacedName(_) => {
                HANDLER.with(|handler| {
                    handler
                        .struct_span_err(name_span, "JSX Namespace is disabled")
                        .emit()
                });

                return Expr::Invalid(Invalid { span: DUMMY_SP });
            }
            JSXElementName::JSXMemberExpr(JSXMemberExpr { obj, prop, .. }) => {
                vnode_kind = Component;
                mut_flags = VNodeFlags::ComponentUnknown as u16;

                fn convert_obj(obj: JSXObject) -> Box<Expr> {
                    let span = obj.span();

                    (match obj {
                        JSXObject::Ident(i) => {
                            if i.sym == "this" {
                                Expr::This(ThisExpr { span })
                            } else {
                                Expr::Ident(i)
                            }
                        }
                        JSXObject::JSXMemberExpr(e) => Expr::Member(MemberExpr {
                            span,
                            obj: convert_obj(e.obj),
                            prop: MemberProp::Ident(e.prop),
                        }),
                    })
                    .into()
                }
                name_expr = Expr::Member(MemberExpr {
                    span: name_span,
                    obj: convert_obj(obj),
                    prop: MemberProp::Ident(prop.clone()),
                })
            }
        }

        let mut props_obj = ObjectLit {
            span: DUMMY_SP,
            props: vec![],
        };

        let mut key_prop = None;
        let mut ref_prop = None;
        let mut component_refs: Option<ObjectLit> = None;

        let mut class_name_param: Option<Box<Expr>> = None;
        let mut has_text_children: bool = false;
        let mut has_keyed_children: bool = false;
        let mut has_non_keyed_children: bool = false;
        let mut children_known: bool = false;
        let mut needs_normalization: bool = false;
        let mut has_re_create_flag: bool = false;
        let mut child_flags_override_param = None;
        let mut flags_override_param = None;
        let mut content_editable_props: bool = false;
        let mut prop_children: Option<Box<Expr>> = None;

        for attr in el.opening.attrs {
            match attr {
                JSXAttrOrSpread::JSXAttr(attr) => {
                    //
                    match attr.name {
                        JSXAttrName::Ident(i) => {
                            //
                            if i.sym == "class" || i.sym == "className" {
                                if vnode_kind == VNodeType::Element {
                                    if let Some(v) = attr.value {
                                        class_name_param =
                                            Some(jsx_attr_value_to_expr_or_invalid(v, i.span))
                                    }

                                    continue;
                                }
                            } else if i.sym == "onDoubleClick" {
                                props_obj
                                    .props
                                    .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                        KeyValueProp {
                                            key: PropName::Ident(IdentName::new(
                                                "onDblClick".into(),
                                                span,
                                            )),
                                            value: match attr.value {
                                                Some(v) => {
                                                    jsx_attr_value_to_expr_or_invalid(v, i.span)
                                                }
                                                None => true.into(),
                                            },
                                        },
                                    ))));
                                continue;
                            } else if i.sym == "key" {
                                key_prop = attr
                                    .value
                                    .and_then(jsx_attr_value_to_expr)
                                    .map(|expr| expr.as_arg());

                                if key_prop.is_none() {
                                    HANDLER.with(|handler| {
                                        handler
                                            .struct_span_err(
                                                i.span,
                                                "The value of property 'key' should not be \
                                                     empty",
                                            )
                                            .emit();
                                    });
                                }

                                continue;
                            } else if i.sym == "ref" {
                                ref_prop = attr
                                    .value
                                    .and_then(jsx_attr_value_to_expr)
                                    .map(|expr| expr.as_arg());

                                if ref_prop.is_none() {
                                    HANDLER.with(|handler| {
                                        handler
                                            .struct_span_err(
                                                i.span,
                                                "The value of property 'ref' should not be \
                                                     empty",
                                            )
                                            .emit();
                                    });
                                }

                                continue;
                            } else if i.sym == "$ChildFlag" {
                                child_flags_override_param = attr
                                    .value
                                    .and_then(jsx_attr_value_to_expr)
                                    .map(|expr| expr.as_arg());

                                if child_flags_override_param.is_none() {
                                    HANDLER.with(|handler| {
                                        handler
                                            .struct_span_err(
                                                i.span,
                                                "The value of property '$ChildFlag' should \
                                                     not be empty",
                                            )
                                            .emit();
                                    });
                                }

                                children_known = true;
                                continue;
                            } else if i.sym == "$HasVNodeChildren" {
                                children_known = true;
                                continue;
                            } else if i.sym == "$Flags" {
                                flags_override_param = attr
                                    .value
                                    .and_then(jsx_attr_value_to_expr)
                                    .map(|expr| expr.as_arg());

                                if flags_override_param.is_none() {
                                    HANDLER.with(|handler| {
                                        handler
                                            .struct_span_err(
                                                i.span,
                                                "The value of property '$Flags' should not be \
                                                     empty",
                                            )
                                            .emit();
                                    });
                                }

                                continue;
                            } else if i.sym == "$HasTextChildren" {
                                children_known = true;
                                has_text_children = true;
                                continue;
                            } else if i.sym == "$HasNonKeyedChildren" {
                                children_known = true;
                                has_non_keyed_children = true;
                                continue;
                            } else if i.sym == "$HasKeyedChildren" {
                                children_known = true;
                                has_keyed_children = true;
                                continue;
                            } else if i.sym == "$ReCreate" {
                                has_re_create_flag = true;
                                continue;
                            }

                            if i.sym.to_ascii_lowercase() == "contenteditable" {
                                content_editable_props = true;
                            } else if i.sym == "children" {
                                if !el.children.is_empty() {
                                    // prop children is ignored if there are any nested children
                                    continue;
                                }

                                prop_children = match attr.value {
                                    Some(v) => Some(jsx_attr_value_to_expr_or_invalid(v, i.span)),
                                    None => continue,
                                };

                                continue;
                            } else if vnode_kind == Component
                                && i.sym.as_ref().starts_with("onComponent")
                            {
                                if let Some(v) = attr.value {
                                    if component_refs.is_none() {
                                        component_refs = Some(ObjectLit {
                                            span: DUMMY_SP,
                                            props: vec![],
                                        })
                                    };

                                    if let Some(some_component_refs) = component_refs.as_mut() {
                                        let ident_span = i.span;
                                        let key = PropName::Ident(i);
                                        let value =
                                            jsx_attr_value_to_expr_or_invalid(v, ident_span);
                                        some_component_refs.props.push(PropOrSpread::Prop(
                                            Box::new(Prop::KeyValue(KeyValueProp { key, value })),
                                        ));
                                    }
                                };

                                continue;
                            }

                            let value = match attr.value {
                                Some(v) => jsx_attr_value_to_expr_or_invalid(v, i.span),
                                None => true.into(),
                            };

                            let converted_prop_name = if vnode_kind == VNodeType::Element
                                && requires_lowercasing(&i.sym)
                            {
                                PropName::Ident(IdentName {
                                    span: i.span,
                                    sym: i.sym.to_lowercase().into(),
                                })
                            } else {
                                let converted_sym = if vnode_kind == VNodeType::Element {
                                    transform_attribute(&i.sym)
                                } else {
                                    &i.sym
                                };

                                if converted_sym.contains('-') || converted_sym.contains(':') {
                                    PropName::Str(Str {
                                        span: i.span,
                                        raw: None,
                                        value: converted_sym.into(),
                                    })
                                } else {
                                    PropName::Ident(IdentName {
                                        span: i.span,
                                        sym: converted_sym.into(),
                                    })
                                }
                            };

                            props_obj
                                .props
                                .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                    key: converted_prop_name,
                                    value,
                                }))));
                        }
                        JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns, name, .. }) => {
                            let value = match attr.value {
                                Some(v) => jsx_attr_value_to_expr_or_invalid(v, ns.span),
                                None => true.into(),
                            };

                            let mut str_value =
                                String::with_capacity(ns.sym.len() + 1 + name.sym.len());
                            str_value.push_str(ns.sym.as_ref());
                            str_value.push(':');
                            str_value.push_str(name.sym.as_ref());
                            let key = Str {
                                span,
                                raw: None,
                                value: str_value.into(),
                            };
                            let key = PropName::Str(key);

                            props_obj
                                .props
                                .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                    key,
                                    value,
                                }))));
                        }
                    }
                }
                JSXAttrOrSpread::SpreadElement(attr) => match *attr.expr {
                    Expr::Object(obj) => {
                        needs_normalization = true;
                        props_obj.props.extend(obj.props);
                    }
                    _ => {
                        needs_normalization = true;
                        props_obj.props.push(PropOrSpread::Spread(attr));
                    }
                },
            }
        }

        let mut children_requires_normalization: bool = false;
        let mut children_found_text: bool = false;
        let mut parent_can_be_keyed: bool = false;
        let mut children_count: u16 = 0;

        let mut children = vec![];

        for child in el.children {
            let child_expr = Some(match child {
                JSXElementChild::JSXText(text) => {
                    // TODO(kdy1): Optimize
                    let value = jsx_text_to_str(&*text.value);
                    let s = Str {
                        span: text.span,
                        raw: None,
                        value,
                    };

                    if s.value.is_empty() {
                        continue;
                    }

                    if vnode_kind == VNodeType::Fragment {
                        ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Call(CallExpr {
                                span: DUMMY_SP,
                                ctxt: unresolved_ctxt,
                                callee: self
                                    .import_create_text_vnode
                                    .get_or_insert_with(|| quote_ident!("createTextVNode").into())
                                    .clone()
                                    .as_callee(),
                                args: vec![s.as_arg()],
                                type_args: Default::default(),
                            })),
                        }
                    } else {
                        children_found_text = true;
                        Lit::Str(s).as_arg()
                    }
                }
                JSXElementChild::JSXExprContainer(JSXExprContainer {
                    expr: JSXExpr::Expr(e),
                    ..
                }) => {
                    children_requires_normalization = true;
                    parent_can_be_keyed = false;
                    e.as_arg()
                }
                JSXElementChild::JSXExprContainer(JSXExprContainer {
                    expr: JSXExpr::JSXEmptyExpr(..),
                    ..
                }) => continue,
                JSXElementChild::JSXElement(el) => {
                    if vnode_kind != Component
                        && !parent_can_be_keyed
                        && !children_known
                        && !children_requires_normalization
                    {
                        // Loop direct children to check if they have key property set
                        parent_can_be_keyed = Self::does_children_have_key_defined(&el);
                    }
                    self.jsx_elem_to_expr(*el).as_arg()
                }
                JSXElementChild::JSXFragment(el) => self.jsx_frag_to_expr(el).as_arg(),
                JSXElementChild::JSXSpreadChild(JSXSpreadChild { span, expr, .. }) => {
                    ExprOrSpread {
                        spread: Some(span),
                        expr,
                    }
                }
            });

            children_count += 1;

            children.push(child_expr)
        }

        if children_found_text {
            match children_count {
                1 => has_text_children = true,
                _ => {
                    for child in &mut children {
                        let Some(expr) = child.take() else {
                            continue;
                        };

                        if let Expr::Lit(Lit::Str(text)) = &*expr.expr {
                            let text = text.clone();
                            *child = Some(ExprOrSpread {
                                spread: None,
                                expr: Box::new(Expr::Call(CallExpr {
                                    span: DUMMY_SP,
                                    ctxt: unresolved_ctxt,
                                    callee: self
                                        .import_create_text_vnode
                                        .get_or_insert_with(|| {
                                            quote_ident!("createTextVNode").into()
                                        })
                                        .clone()
                                        .as_callee(),
                                    args: vec![text.as_arg()],
                                    type_args: Default::default(),
                                })),
                            });
                        } else {
                            *child = Some(expr);
                        }
                    }
                }
            }
        }

        parent_can_be_keyed =
            children_count > 1 && parent_can_be_keyed && !children_requires_normalization;
        let parent_can_be_non_keyed =
            children_count > 1 && !parent_can_be_keyed && !children_requires_normalization;

        let child_flags: ChildFlags;

        if !children_requires_normalization || children_known {
            if has_keyed_children || parent_can_be_keyed {
                child_flags = ChildFlags::HasKeyedChildren;
            } else if has_non_keyed_children || parent_can_be_non_keyed {
                child_flags = ChildFlags::HasNonKeyedChildren;
            } else if children_count == 1 {
                if has_text_children {
                    child_flags = ChildFlags::HasTextChildren;
                } else if vnode_kind == VNodeType::Fragment {
                    child_flags = ChildFlags::HasNonKeyedChildren;
                } else {
                    child_flags = ChildFlags::HasVNodeChildren;
                }
            } else {
                child_flags = ChildFlags::HasInvalidChildren
            }
        } else if has_keyed_children {
            child_flags = ChildFlags::HasKeyedChildren;
        } else if has_non_keyed_children {
            child_flags = ChildFlags::HasNonKeyedChildren;
        } else if has_text_children {
            child_flags = ChildFlags::HasTextChildren;
        } else {
            child_flags = ChildFlags::UnknownChildren;
        }

        if vnode_kind == Component {
            match children.len() {
                0 => {
                    match prop_children {
                        Some(some_prop_children) => {
                            props_obj
                                .props
                                .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                    key: PropName::Ident(quote_ident!("children")),
                                    value: some_prop_children,
                                }))));
                        }
                        None => {
                            // noop
                        }
                    }
                }
                1 => {
                    if let Some(Some(ExprOrSpread { spread: None, .. })) = children.first() {
                        if let Some(child) = children.take().into_iter().next().flatten() {
                            props_obj
                                .props
                                .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                    key: PropName::Ident(quote_ident!("children")),
                                    value: child.expr,
                                }))));
                        }
                    } else {
                        props_obj
                            .props
                            .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                key: PropName::Ident(quote_ident!("children")),
                                value: Box::new(Expr::Array(ArrayLit {
                                    span: DUMMY_SP,
                                    elems: children.take(),
                                })),
                            }))));
                    }
                }
                _ => {
                    props_obj
                        .props
                        .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                            key: PropName::Ident(quote_ident!("children")),
                            value: Box::new(Expr::Array(ArrayLit {
                                span: DUMMY_SP,
                                elems: children.take(),
                            })),
                        }))));
                }
            }
        } else {
            // Backwards compatibility...
            // Set prop children as children if no nested children were set
            if children.is_empty() {
                match prop_children {
                    Some(some_prop_children) => children.push(Some(ExprOrSpread {
                        spread: None,
                        expr: some_prop_children,
                    })),
                    None => {
                        // noop
                    }
                }
            }
        }

        self.top_level_node = top_level_node;

        if has_re_create_flag {
            mut_flags |= VNodeFlags::ReCreate as u16;
        }
        if content_editable_props {
            mut_flags |= VNodeFlags::ContentEditable as u16;
        }

        let flags_expr = match flags_override_param {
            None => Box::new(Expr::Lit(Lit::Num(Number {
                span: DUMMY_SP,
                raw: None,
                value: mut_flags as f64,
            })))
            .as_arg(),
            Some(v) => v,
        };

        let create_method = if vnode_kind == Component {
            self.import_create_component
                .get_or_insert_with(|| quote_ident!("createComponentVNode").into())
                .clone()
        } else if vnode_kind == VNodeType::Element {
            self.import_create_vnode
                .get_or_insert_with(|| quote_ident!("createVNode").into())
                .clone()
        } else {
            self.import_create_fragment
                .get_or_insert_with(|| quote_ident!("createFragment").into())
                .clone()
        };

        let create_method_args = if vnode_kind == Component {
            // Functional component cannot have basic ref so when component refs is set use it
            // If we can ever detect Functional component from Class component compile time
            // We could add some validations
            if let Some(some_refs) = component_refs {
                create_component_vnode_args(
                    flags_expr,
                    name_expr,
                    props_obj,
                    key_prop,
                    Some(some_refs.as_arg()),
                )
            } else {
                create_component_vnode_args(flags_expr, name_expr, props_obj, key_prop, ref_prop)
            }
        } else if vnode_kind == VNodeType::Element {
            CreateVNodeArgs {
                flags: flags_expr,
                name: name_expr,
                class_name: class_name_param,
                children,
                child_flags: child_flags as u16,
                child_flags_override_param,
                props: props_obj,
                key: key_prop,
                refs: ref_prop,
            }
            .into_args()
        } else {
            create_fragment_vnode_args(
                children,
                has_non_keyed_children
                    || has_keyed_children
                    || child_flags_override_param.is_some(),
                child_flags as u16,
                child_flags_override_param,
                key_prop,
            )
        };

        let create_expr = Expr::Call(CallExpr {
            span,
            ctxt: unresolved_ctxt,
            callee: create_method.as_callee(),
            args: create_method_args,
            type_args: Default::default(),
        });

        if needs_normalization {
            return Expr::Call(CallExpr {
                span,
                ctxt: unresolved_ctxt,
                callee: self
                    .import_normalize_props
                    .get_or_insert_with(|| quote_ident!("normalizeProps").into())
                    .clone()
                    .as_callee(),
                args: vec![create_expr.as_arg()],
                type_args: Default::default(),
            });
        }

        create_expr
    }

    fn does_children_have_key_defined(el: &JSXElement) -> bool {
        for attr in &el.opening.attrs {
            match attr {
                JSXAttrOrSpread::JSXAttr(attr) => {
                    //
                    match &attr.name {
                        JSXAttrName::Ident(i) => {
                            if i.sym == "key" {
                                return true;
                            }
                        }
                        JSXAttrName::JSXNamespacedName(_) => {
                            continue;
                        }
                    }
                }
                JSXAttrOrSpread::SpreadElement(_attr) => {
                    continue;
                }
            }
        }

        false
    }
}

impl<C> VisitMut for Jsx<C>
where
    C: Comments,
{
    noop_visit_mut_type!();

    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        let top_level_node = self.top_level_node;
        let mut did_work = false;

        if let Expr::JSXElement(el) = expr {
            did_work = true;
            // <div></div> => Inferno.createVNode(...);
            *expr = self.jsx_elem_to_expr(*el.take());
        } else if let Expr::JSXFragment(frag) = expr {
            // <></> => Inferno.createFragment(...);
            did_work = true;
            *expr = self.jsx_frag_to_expr(frag.take());
        } else if let Expr::Paren(ParenExpr {
            expr: inner_expr, ..
        }) = expr
        {
            if let Expr::JSXElement(el) = &mut **inner_expr {
                did_work = true;
                *expr = self.jsx_elem_to_expr(*el.take());
            } else if let Expr::JSXFragment(frag) = &mut **inner_expr {
                // <></> => Inferno.createFragment(...);
                did_work = true;
                *expr = self.jsx_frag_to_expr(frag.take());
            }
        }

        if did_work {
            self.top_level_node = false;
        }

        expr.visit_mut_children_with(self);

        self.top_level_node = top_level_node;
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        self.set_local_import_refs(&mut module.body);

        module.visit_mut_children_with(self);

        self.inject_runtime(&mut module.body, |imports, default_import_src, stmts| {
            // Merge new imports to existing import
            if merge_imports(&imports, &default_import_src, stmts) {
                return;
            }

            // Existing inferno import was not found, add new
            let specifiers: Vec<ImportSpecifier> = imports
                .into_iter()
                .map(|imported| {
                    ImportSpecifier::Named(ImportNamedSpecifier {
                        span: DUMMY_SP,
                        local: imported,
                        imported: None,
                        is_type_only: false,
                    })
                })
                .collect();

            prepend_stmt(
                stmts,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    span: DUMMY_SP,
                    specifiers,
                    src: Str {
                        span: DUMMY_SP,
                        raw: None,
                        value: default_import_src.clone(),
                    }
                    .into(),
                    type_only: Default::default(),
                    with: Default::default(),
                    phase: Default::default(),
                })),
            )
        });
    }

    fn visit_mut_script(&mut self, script: &mut Script) {
        script.visit_mut_children_with(self);

        let mark = self.unresolved_mark;
        self.inject_runtime(&mut script.body, |imports, src, stmts| {
            prepend_stmt(stmts, add_require(imports, src, mark))
        });
    }
}

#[inline]
fn is_component_vnode(i: &Ident) -> bool {
    // If it starts with uppercase
    i.as_ref().starts_with(|c: char| c.is_ascii_uppercase())
}

// const { createElement } = require('react')
// const { jsx: jsx } = require('react/jsx-runtime')
fn add_require(imports: Vec<Ident>, src: Wtf8Atom, unresolved_mark: Mark) -> Stmt {
    VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Const,
        declare: false,
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Object(ObjectPat {
                span: DUMMY_SP,
                props: imports
                    .into_iter()
                    .map(|local| {
                        ObjectPatProp::Assign(AssignPatProp {
                            span: DUMMY_SP,
                            key: local.into(),
                            value: None,
                        })
                    })
                    .collect(),
                optional: false,
                type_ann: None,
            }),
            // require('react')
            init: Some(Box::new(Expr::Call(CallExpr {
                span: DUMMY_SP,
                callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                    ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
                    sym: atom!("require"),
                    optional: false,
                    ..Default::default()
                }))),
                args: vec![ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Lit(Lit::Str(Str {
                        span: DUMMY_SP,
                        value: src,
                        raw: None,
                    }))),
                }],
                ..Default::default()
            }))),
            definite: false,
        }],
        ..Default::default()
    }
    .into()
}
