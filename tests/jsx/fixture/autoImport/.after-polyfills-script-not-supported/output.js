var _reactJsxRuntime = require("react/jsx-runtime");

// https://github.com/babel/babel/issues/12522
require('app-polyfill/ie11');

require('app-polyfill/stable');

const Inferno = require('react-dom');

Inferno.render( /*#__PURE__*/_reactJsxRuntime.jsx("p", {
  children: "Hello, World!"
}), document.getElementById('root'));
