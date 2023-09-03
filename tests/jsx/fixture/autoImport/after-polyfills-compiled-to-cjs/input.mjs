// https://github.com/babel/babel/issues/12522

import "app-polyfill/ie11";
import "app-polyfill/stable";
import Inferno from "inferno";

Inferno.render(<p>Hello, World!</p>, document.getElementById("root"));
