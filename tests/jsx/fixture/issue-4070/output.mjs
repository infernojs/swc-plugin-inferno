import { newVNode, newComponentVNode, normalizeProps } from "inferno";
const ChildrenFail = (props)=>{
    return array.map((label)=>/*#__PURE__*/ normalizeProps(/*#__PURE__*/ newComponentVNode(0, WrapperWhereMagicHappens, {
            ...props,
            children: /*#__PURE__*/ newVNode(1, "h2", null, label)
        }, label)));
};
