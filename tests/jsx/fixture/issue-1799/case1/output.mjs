// Foo.jsx
import Inferno, { createVNode } from "inferno";
export default function Foo() {
    return /*#__PURE__*/ createVNode(1, "div", null, null, 1, {
        onClick: async (e)=>{
            await doSomething();
        }
    });
}
Foo.displayName = "Foo";
