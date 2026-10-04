import { newVNode, newComponentVNode } from "inferno";
var x = /*#__PURE__*/ newVNode(5, "div", null, [
    /*#__PURE__*/ newVNode(9, "div", null, /*#__PURE__*/ newVNode(17, "br")),
    /*#__PURE__*/ newComponentVNode(0, Component, {
        children: [
            foo,
            /*#__PURE__*/ newVNode(17, "br"),
            bar
        ]
    }),
    /*#__PURE__*/ newVNode(17, "br")
]);
