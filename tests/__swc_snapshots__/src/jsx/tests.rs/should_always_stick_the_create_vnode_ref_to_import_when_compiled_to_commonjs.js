import { Component, createTextVNode, createVNode, linkEvent, render, createComponentVNode } from 'inferno';
const Foo = class Clock extends Component {
    public render() {
        return /*#__PURE__*/ createComponentVNode(2, Collapsible, {
            children: [
                /*#__PURE__*/ createVNode(1, "div", null, [
                    [
                        /*#__PURE__*/ createVNode(1, "p", null, "Hello 0", 16),
                        /*#__PURE__*/ createVNode(1, "p", null, "Hello 1", 16)
                    ],
                    /*#__PURE__*/ createVNode(1, "strong", null, "Hello 2", 16)
                ], 0),
                /*#__PURE__*/ createVNode(1, "p", null, "Hello 3", 16)
            ]
        });
    }
};
render(/*#__PURE__*/ createComponentVNode(2, Foo), null);
