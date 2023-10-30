import { createVNode, createComponentVNode } from "inferno";
var x = /*#__PURE__*/ createVNode(1, "div", null, [
    /*#__PURE__*/ createVNode(1, "div", null, /*#__PURE__*/ createVNode(1, "br"), 2),
    /*#__PURE__*/ createComponentVNode(2, Component, {
        children: [
            foo,
            /*#__PURE__*/ createVNode(1, "br"),
            bar
        ]
    }),
    /*#__PURE__*/ createVNode(1, "br")
], 4);
