import { newVNode } from "inferno";
var x = /*#__PURE__*/ newVNode(9, "div", null, /*#__PURE__*/ newVNode(17, "span" // a double-slash comment
, null, null, {
    attr2: "bar"
}), {
    /* a multi-line
       comment */ attr1: "foo"
});
