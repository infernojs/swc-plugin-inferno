import { createVNode, normalizeProps } from "inferno";
import * as inferno from "inferno";
var y = /*#__PURE__*/ inferno.createElement("div", {
    foo: 1
});
var x = /*#__PURE__*/ createVNode(1, "div", null, [
    /*#__PURE__*/ createVNode(1, "div", null, null, 1, null, "1"),
    /*#__PURE__*/ createVNode(1, "div", null, null, 1, {
        meow: "wolf"
    }, "2"),
    /*#__PURE__*/ createVNode(1, "div", null, null, 1, null, "3"),
    /*#__PURE__*/ normalizeProps(createVNode(1, "div", null, null, 1, {
        ...props
    }, "4"))
], 8);
