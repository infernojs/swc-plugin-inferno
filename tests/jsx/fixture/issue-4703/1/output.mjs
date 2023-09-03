/** @jsx foo */ import { createVNode } from "inferno";
function ProductItem() {
    return /*#__PURE__*/ createVNode(1, "div", null, "Hello World", 16);
}
console.log(ProductItem);
