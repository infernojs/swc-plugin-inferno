import { newVNode, newComponentVNode, newTextVNode } from "inferno";
var HelloMessage = Inferno.createClass({
    render: function() {
        return /*#__PURE__*/ newVNode(1, "div", null, [
            /*#__PURE__*/ newTextVNode("Hello "),
            this.props.name
        ]);
    }
});
Inferno.render(/*#__PURE__*/ newComponentVNode(0, HelloMessage, {
    name: /*#__PURE__*/ newVNode(3, "span", null, "Sebastian")
}), mountNode);
