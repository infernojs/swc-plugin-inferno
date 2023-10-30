import { createVNode } from "inferno";
var x = /*#__PURE__*/ createVNode(1, "div", null, /*#__PURE__*/ createVNode(1, "span" // a double-slash comment
, null, null, 1, {
    attr2: "bar"
}), 2, {
    /* a multi-line
comment */ attr1: "foo"
});
