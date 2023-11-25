/** @jsxRuntime classic */ import { createComponentVNode, createTextVNode } from "inferno";
class MobileHomeActivityTaskPriorityIcon extends Inferno.PureComponent {
    render() {
        return /*#__PURE__*/ createComponentVNode(2, Text, {
            children: [
                /*#__PURE__*/ createTextVNode(" "),
                this.props.value,
                /*#__PURE__*/ createTextVNode(" ")
            ]
        });
    }
}
