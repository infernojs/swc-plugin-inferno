import { newVNode, newComponentVNode } from "inferno";
var foo = function() {
    return ()=>/*#__PURE__*/ newVNode(17, "this");
};
var bar = function() {
    return ()=>/*#__PURE__*/ newComponentVNode(0, this.foo);
};
