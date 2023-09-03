/** @jsx dom */ /*#__PURE__*/ import { createVNode, createComponentVNode } from "inferno";
createComponentVNode(2, Foo);
var profile = /*#__PURE__*/ createVNode(1, "div", null, [
    /*#__PURE__*/ createVNode(1, "img", "profile", null, 1, {
        src: "avatar.png"
    }),
    /*#__PURE__*/ createVNode(1, "h3", null, [
        user.firstName,
        user.lastName
    ].join(" "), 0)
], 4);
