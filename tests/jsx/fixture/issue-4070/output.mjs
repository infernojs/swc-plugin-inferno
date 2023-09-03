import { createVNode, createComponentVNode, normalizeProps } from "inferno";
const ChildrenFail = (props)=>{
    return array.map((label)=>/*#__PURE__*/ normalizeProps(createComponentVNode(2, WrapperWhereMagicHappens, {
        ...props,
        children: /*#__PURE__*/ createVNode(1, "h2", null, label, 0)
    }, label)));
};
