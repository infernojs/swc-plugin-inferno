import { createComponentVNode, normalizeProps } from "inferno";
/*#__PURE__*/ normalizeProps(createComponentVNode(2, Component, {
    y: 2,
    ...x,
    z: true
}));
