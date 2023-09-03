/*#__PURE__*/ import { createVNode, normalizeProps } from "inferno";
createVNode(1, "p", null, "text", 16, {
    prop: true,
    prop: true
});
/*#__PURE__*/ normalizeProps(createVNode(1, "p", null, "text", 16, {
    prop,
    prop
}));
/*#__PURE__*/ normalizeProps(createVNode(1, "p", null, "text", 16, {
    prop: true,
    prop
}));
/*#__PURE__*/ normalizeProps(createVNode(1, "p", null, "text", 16, {
    prop,
    prop: true
}));
