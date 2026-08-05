import { Component, createVNode, createComponentVNode } from "inferno";
export default class App extends Component {
    render() {
        return /*#__PURE__*/ createComponentVNode(2, Main);
    }
}
export const Title = ()=>/*#__PURE__*/ createVNode(1, "h1", null, "App", 16);
