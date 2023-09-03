import { createVNode, createComponentVNode } from "inferno";
import _JSXStyle from "styled-jsx/style";
const WithSidebar = ({ right = false, top = false, sidebar, sidebarWidth = 230, hideOnMobile = false, breakpoint = 730, children })=>/*#__PURE__*/ createVNode(1, "main", _JSXStyle.dynamic([
    [
        "4507deac72c40d6c",
        [
            right ? "row-reverse" : "row",
            sidebarWidth,
            breakpoint,
            top ? "column" : "column-reverse"
        ]
    ]
]), [
    /*#__PURE__*/ createComponentVNode(2, Sidebar, {
        width: sidebarWidth,
        right: right,
        hide: hideOnMobile,
        breakpoint: breakpoint,
        children: sidebar
    }),
    /*#__PURE__*/ createVNode(1, "div", _JSXStyle.dynamic([
        [
            "4507deac72c40d6c",
            [
                right ? "row-reverse" : "row",
                sidebarWidth,
                breakpoint,
                top ? "column" : "column-reverse"
            ]
        ]
    ]), children, 0),
    /*#__PURE__*/ createVNode(1, "_JSXStyle", null, `main.__jsx-style-dynamic-selector{display:-webkit-box;display:-webkit-flex;display:-moz-box;display:-ms-flexbox;display:flex;-webkit-flex-direction:${right ? "row-reverse" : "row"};-ms-flex-direction:${right ? "row-reverse" : "row"};flex-direction:${right ? "row-reverse" : "row"};-webkit-box-pack:justify;-webkit-justify-content:space-between;-moz-box-pack:justify;-ms-flex-pack:justify;justify-content:space-between;margin-bottom:var(--geist-gap-double)}div.__jsx-style-dynamic-selector{width:100%;max-width:-webkit-calc(100% - ${sidebarWidth}px - var(--geist-gap-double));max-width:-moz-calc(100% - ${sidebarWidth}px - var(--geist-gap-double));max-width:calc(100% - ${sidebarWidth}px - var(--geist-gap-double))}@media(max-width:${breakpoint}px){main.__jsx-style-dynamic-selector{-webkit-flex-direction:${top ? "column" : "column-reverse"};-ms-flex-direction:${top ? "column" : "column-reverse"};flex-direction:${top ? "column" : "column-reverse"}}div.__jsx-style-dynamic-selector{max-width:unset}}`, 0, {
        id: "4507deac72c40d6c",
        dynamic: [
            right ? "row-reverse" : "row",
            sidebarWidth,
            breakpoint,
            top ? "column" : "column-reverse"
        ]
    })
], 4);
