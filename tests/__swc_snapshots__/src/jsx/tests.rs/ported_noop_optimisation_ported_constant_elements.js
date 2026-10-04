import { newVNode } from "inferno";
import { Component } from "inferno";
class App extends Component {
    render() {
        const navbarHeader = /*#__PURE__*/ newVNode(9, "div", "navbar-header", /*#__PURE__*/ newVNode(9, "a", "navbar-brand", /*#__PURE__*/ newVNode(17, "img", null, null, {
            src: "/img/logo/logo-96x36.png"
        }), {
            href: "/"
        }));
        return /*#__PURE__*/ newVNode(9, "div", null, /*#__PURE__*/ newVNode(9, "nav", "navbar navbar-default", /*#__PURE__*/ newVNode(1, "div", "container", navbarHeader)));
    }
}
