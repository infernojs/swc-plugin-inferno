import { createComponentVNode } from "inferno";
var foo = function() {
    return ()=>/*#__PURE__*/ createComponentVNode(2, this);
};
var bar = function() {
    return ()=>/*#__PURE__*/ createComponentVNode(2, this.foo);
};
