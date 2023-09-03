/** @jsxRuntime classic */ import { createComponentVNode, createTextVNode } from "inferno";
class MobileHomeActivityTaskPriorityIcon extends Inferno.PureComponent {
    render() {
        return /*#__PURE__*/ createComponentVNode(2, Text, {
            children: [
                createTextVNode("\xa0"),
                this.props.value,
                createTextVNode("\xa0")
            ]
        });
    }
}
