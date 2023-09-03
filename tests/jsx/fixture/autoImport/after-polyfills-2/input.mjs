// https://github.com/babel/babel/issues/12522

Inferno.render(<p>Hello, World!</p>, document.getElementById("root"));

// Imports are hoisted, so this is still ok
import "app-polyfill/ie11";
import "app-polyfill/stable";
import Inferno from "inferno";
