import { createVNode } from "inferno";
/*#__PURE__*/ createVNode(1, "div", null, "this should not parse as unicode: \\u00a0", 16);
