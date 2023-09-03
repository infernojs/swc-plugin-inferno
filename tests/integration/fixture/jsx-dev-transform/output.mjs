import { createVNode, createFragment } from "inferno";
const App = /*#__PURE__*/ createVNode(1, "div", null, [
    /*#__PURE__*/ createVNode(1, "div"),
    /*#__PURE__*/ createFragment([
        /*#__PURE__*/ createVNode(1, "div", null, "hoge", 16, null, 1)
    ], 8)
], 4);
