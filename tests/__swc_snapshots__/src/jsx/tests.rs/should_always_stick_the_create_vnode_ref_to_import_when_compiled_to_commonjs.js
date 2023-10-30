"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _inferno = require("inferno");
const Foo = class Clock extends _inferno.Component {
    public render() {
        return /*#__PURE__*/ (0, _inferno.createComponentVNode)(2, Collapsible, {
            children: [
                /*#__PURE__*/ (0, _inferno.createVNode)(1, "div", null, [
                    [
                        /*#__PURE__*/ (0, _inferno.createVNode)(1, "p", null, "Hello 0", 16),
                        /*#__PURE__*/ (0, _inferno.createVNode)(1, "p", null, "Hello 1", 16)
                    ],
                    /*#__PURE__*/ (0, _inferno.createVNode)(1, "strong", null, "Hello 2", 16)
                ], 0),
                /*#__PURE__*/ (0, _inferno.createVNode)(1, "p", null, "Hello 3", 16)
            ]
        });
    }
};
(0, _inferno.render)(/*#__PURE__*/ (0, _inferno.createComponentVNode)(2, Foo), null);
