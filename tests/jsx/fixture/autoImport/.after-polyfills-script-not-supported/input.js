// https://github.com/babel/babel/issues/12522

require("app-polyfill/ie11");
require("app-polyfill/stable");
const Inferno = require("inferno");

Inferno.render(<p>Hello, World!</p>, document.getElementById("root"));
