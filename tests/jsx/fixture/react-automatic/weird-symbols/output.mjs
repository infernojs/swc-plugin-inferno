import { createComponentVNode, createTextVNode } from "inferno";
class MobileHomeActivityTaskPriorityIcon extends Inferno.PureComponent {
    render() {
        return /*#__PURE__*/ createComponentVNode(2, Text, {
            children: [
                createTextVNode(" "),
                this.props.value,
                createTextVNode(" ")
            ]
        });
    }
}
