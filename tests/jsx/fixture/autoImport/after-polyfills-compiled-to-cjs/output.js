"use strict";

require("app-polyfill/ie11");

require("app-polyfill/stable");

var _Inferno = _interop_require_default(require("inferno"));

var _jsxRuntime = require("react/jsx-runtime");

function _interop_require_default(obj) { return obj && obj.__esModule ? obj : { default: obj }; }

// https://github.com/babel/babel/issues/12522
_Inferno.default.render( /*#__PURE__*/(0, _jsxRuntime.jsx)("p", {
  children: "Hello, World!"
}), document.getElementById('root'));
