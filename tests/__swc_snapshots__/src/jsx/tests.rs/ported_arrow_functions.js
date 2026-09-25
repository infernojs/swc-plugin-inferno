import { createVNode, createComponentVNode } from "inferno";
var foo = function() {
    return ()=>/*#__PURE__*/ createVNode(1, "this");
};
var bar = function() {
    return ()=>/*#__PURE__*/ createComponentVNode(2, this.foo);
};
