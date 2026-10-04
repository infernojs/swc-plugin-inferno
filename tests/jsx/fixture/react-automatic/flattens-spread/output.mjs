import { newVNode, normalizeProps } from "inferno";
/*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(3, "p", null, "text", {
    ...props
}));
/*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(1, "div", null, contents, {
    ...props
}));
/*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(17, "img", null, null, {
    alt: "",
    ...{
        src,
        title
    }
}));
/*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(1, "blockquote", null, items, {
    ...{
        cite
    }
}));
