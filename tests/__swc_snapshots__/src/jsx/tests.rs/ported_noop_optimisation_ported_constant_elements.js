import { Component, createVNode } from "inferno";
class App extends Component {
    render() {
        const navbarHeader = /*#__PURE__*/ createVNode(1, "div", "navbar-header", /*#__PURE__*/ createVNode(1, "a", "navbar-brand", /*#__PURE__*/ createVNode(1, "img", null, null, 1, {
            src: "/img/logo/logo-96x36.png"
        }), 2, {
            href: "/"
        }), 2);
        return /*#__PURE__*/ createVNode(1, "div", null, /*#__PURE__*/ createVNode(1, "nav", "navbar navbar-default", /*#__PURE__*/ createVNode(1, "div", "container", navbarHeader, 0), 2), 2);
    }
}
