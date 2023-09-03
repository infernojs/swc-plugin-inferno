import { createComponentVNode } from "inferno";
export default (()=>{
    return /*#__PURE__*/ createComponentVNode(2, Input, {
        pattern: ".*\\S+.*"
    });
});
