import { newVNode, newComponentVNode } from "inferno";
import { Component } from "inferno";
export default class App extends Component {
    render() {
        return /*#__PURE__*/ newComponentVNode(0, Main);
    }
}
export const Title = ()=>/*#__PURE__*/ newVNode(3, "h1", null, "App");
