var _inferno = require("inferno"), newVNode = _inferno.newVNode, newFragment = _inferno.newFragment, normalizeProps = _inferno.normalizeProps;
var x = /*#__PURE__*/ newFragment(260, [
    /*#__PURE__*/ newVNode(33, "div", null, [
        /*#__PURE__*/ newVNode(17, "div", null, null, null, "1"),
        /*#__PURE__*/ newVNode(17, "div", null, null, {
            meow: "wolf"
        }, "2"),
        /*#__PURE__*/ newVNode(17, "div", null, null, null, "3"),
        /*#__PURE__*/ normalizeProps(/*#__PURE__*/ newVNode(17, "div", null, null, {
            ...props
        }, "4"))
    ])
]);
