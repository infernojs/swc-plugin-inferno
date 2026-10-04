import { newVNode, newComponentVNode } from "inferno";
import Inferno from "inferno";
import Inferno from "inferno";
import { Button, Input } from "antd";
import Child from "./component/Child";
class Page extends Inferno.Component {
    render() {
        return /*#__PURE__*/ newVNode(5, "div", "test", [
            /*#__PURE__*/ newVNode(3, "div", null, "Page"),
            /*#__PURE__*/ newComponentVNode(0, Child),
            /*#__PURE__*/ newVNode(528, "input", null, null, {
                placeholder: "我是谁?"
            }),
            /*#__PURE__*/ newComponentVNode(0, Button, {
                children: "click me"
            }),
            /*#__PURE__*/ newComponentVNode(0, Input)
        ]);
    }
}
