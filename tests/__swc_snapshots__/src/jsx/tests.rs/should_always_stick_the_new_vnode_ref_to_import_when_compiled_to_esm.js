import { newComponentVNode } from "inferno";
import { Component, newTextVNode, newVNode, linkEvent, render } from 'inferno';
const Foo = class Clock extends Component {
    public render() {
        return /*#__PURE__*/ newComponentVNode(0, Collapsible, {
            children: [
                /*#__PURE__*/ newVNode(1, "div", null, [
                    [
                        /*#__PURE__*/ newVNode(3, "p", null, "Hello 0"),
                        /*#__PURE__*/ newVNode(3, "p", null, "Hello 1")
                    ],
                    /*#__PURE__*/ newVNode(3, "strong", null, "Hello 2")
                ]),
                /*#__PURE__*/ newVNode(3, "p", null, "Hello 3")
            ]
        });
    }
};
render(/*#__PURE__*/ newComponentVNode(0, Foo), null);
