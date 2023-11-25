import { createVNode, createComponentVNode, createTextVNode } from "inferno";
var HelloMessage = Inferno.createClass({
    render: function() {
        return /*#__PURE__*/ createVNode(1, "div", null, [
            /*#__PURE__*/ createTextVNode("Hello "),
            this.props.name
        ], 0);
    }
});
Inferno.render(/*#__PURE__*/ createComponentVNode(2, HelloMessage, {
    name: /*#__PURE__*/ createVNode(1, "span", null, "Sebastian", 16)
}), mountNode);
