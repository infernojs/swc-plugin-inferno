import { newVNode, newFragment, newTextVNode } from "inferno";
/*#__PURE__*/ newFragment(260, [
    /*#__PURE__*/ newVNode(3, "span", null, "Hello something long to not trigger line break"),
    /*#__PURE__*/ newTextVNode(" ")
]);
