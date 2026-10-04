import { newVNode, normalizeProps } from "inferno";
/*#__PURE__*/ newVNode(3, "p", null, "text", {
    prop: true
});
/*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(3, "p", null, "text", {
    ...{
        prop,
        prop
    }
}));
/*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(3, "p", null, "text", {
    prop: true,
    ...{
        prop
    }
}));
/*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(3, "p", null, "text", {
    ...{
        prop
    },
    prop: true
}));
