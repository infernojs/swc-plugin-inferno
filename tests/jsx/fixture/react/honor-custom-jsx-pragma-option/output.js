import { newVNode, newComponentVNode } from "inferno";
/*#__PURE__*/ newComponentVNode(0, Foo);
var profile = /*#__PURE__*/ newVNode(5, "div", null, [
    /*#__PURE__*/ newVNode(17, "img", "profile", null, {
        src: "avatar.png"
    }),
    /*#__PURE__*/ newVNode(1, "h3", null, [
        user.firstName,
        user.lastName
    ].join(" "))
]);
