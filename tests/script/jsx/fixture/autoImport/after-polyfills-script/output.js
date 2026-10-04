var _inferno = require("inferno"), newVNode = _inferno.newVNode;
// https://github.com/babel/babel/issues/12522
require("app-polyfill/ie11");
require("app-polyfill/stable");
const Inferno = require("inferno");
Inferno.render(/*#__PURE__*/ newVNode(3, "p", null, "Hello, World!"), document.getElementById("root"));
