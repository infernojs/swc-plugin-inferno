/** @jsx h */ /** @jsxFrag */ import { createVNode, createFragment } from "inferno";
import { h } from "preact";
import { Marked } from "markdown";
export const handler = {
    async GET (req, ctx) {
        const markdown = await Deno.readTextFile(`posts/${ctx.params.id}.md`);
        const markup = Marked.parse(markdown);
        const resp = await ctx.render({
            markup
        });
        return resp;
    }
};
export default function Greet(props) {
    return /*#__PURE__*/ createFragment([
        /*#__PURE__*/ createVNode(1, "div", null, null, 1, {
            dangerouslySetInnerHTML: {
                __html: props.data.markup.content
            }
        })
    ], 4);
}
