import { createVNode, normalizeProps } from "@emotion/react";
import { isValidElement, Children } from "inferno";
import * as styles from "./CheckmarkList.styles";
const CheckmarkList = ({ children })=>{
    const listItems = ()=>Children.map(children, (child, index)=>{
            if (!isValidElement(child)) {
                return null;
            }
            const { children: liChildren, css: liCss, ...otherProps } = child.props;
            return /*#__PURE__*/ normalizeProps(createVNode(1, "li", null, liChildren, 0, {
                ...otherProps,
                css: [
                    styles.listItem,
                    liCss
                ]
            }, `checkmark-list-item-${index}`));
        });
    return /*#__PURE__*/ createVNode(1, "ul", null, listItems(), 0, {
        css: styles.list
    });
};
export { CheckmarkList };
