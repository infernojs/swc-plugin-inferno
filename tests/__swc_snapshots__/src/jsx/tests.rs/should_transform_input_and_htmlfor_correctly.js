import { newVNode } from "inferno";
/*#__PURE__*/ newVNode(9, "label", null, /*#__PURE__*/ newVNode(528, "input", null, null, {
    id: id,
    name: name,
    value: value,
    onChange: onChange,
    onInput: onInput,
    onKeyup: onKeyup,
    onFocus: onFocus,
    onClick: onClick,
    type: "number",
    pattern: "[0-9]+([,\\.][0-9]+)?",
    inputmode: "numeric",
    min: minimum
}), {
    for: id
});
