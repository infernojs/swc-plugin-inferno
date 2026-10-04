import { newVNode } from "inferno";
/*#__PURE__*/ newVNode(3, "div", null, "this should not parse as unicode: \\u00a0");
