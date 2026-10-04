import { newVNode, newTextVNode } from "inferno";
let page = /*#__PURE__*/ newVNode(5, "p", null, [
    /*#__PURE__*/ newTextVNode("Click "),
    /*#__PURE__*/ newVNode(3, "em", null, "New melody"),
    /*#__PURE__*/ newTextVNode(" listen to a randomly generated melody")
]);
