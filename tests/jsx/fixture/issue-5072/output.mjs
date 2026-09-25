import { createVNode, createComponentVNode } from "inferno";
import Inferno from "inferno";
import Inferno from "inferno";
import { Button, Input } from "antd";
import Child from "./component/Child";
class Page extends Inferno.Component {
    render() {
        return /*#__PURE__*/ createVNode(1, "div", "test", [
            /*#__PURE__*/ createVNode(1, "div", null, "Page", 16),
            /*#__PURE__*/ createComponentVNode(2, Child),
            /*#__PURE__*/ createVNode(64, "input", null, null, 1, {
                placeholder: "我是谁?"
            }),
            /*#__PURE__*/ createComponentVNode(2, Button, {
                children: "click me"
            }),
            /*#__PURE__*/ createComponentVNode(2, Input)
        ], 4);
    }
}
