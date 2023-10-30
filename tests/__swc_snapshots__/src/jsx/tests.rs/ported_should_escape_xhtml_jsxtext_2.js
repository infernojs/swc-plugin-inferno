/*#__PURE__*/ import { createVNode } from "inferno";
createVNode(1, "div", null, "this should not parse as unicode: \\u00a0", 16);
