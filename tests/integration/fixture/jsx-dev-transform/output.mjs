import { newVNode, newFragment } from "inferno";
const App = /*#__PURE__*/ newVNode(5, "div", null, [
    /*#__PURE__*/ newVNode(17, "div"),
    /*#__PURE__*/ newFragment(260, [
        /*#__PURE__*/ newVNode(3, "div", null, "hoge", null, 1)
    ])
]);
