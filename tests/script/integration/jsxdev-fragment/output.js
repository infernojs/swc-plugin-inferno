const { createVNode, createFragment } = require("inferno");
const App = /*#__PURE__*/ createFragment([
    /*#__PURE__*/ createVNode(1, "div", null, "hoge", 16),
    /*#__PURE__*/ createVNode(1, "div", null, "fuga", 16)
], 4);
