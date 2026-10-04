import { newVNode } from "inferno";
// Foo.jsx
import Inferno from "inferno";
export default function Foo() {
    return /*#__PURE__*/ newVNode(17, "div", null, null, {
        onClick: async (e)=>{
            await doSomething();
        }
    });
}
Foo.displayName = "Foo";
