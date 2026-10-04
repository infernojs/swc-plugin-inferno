import { newVNode, newFragment } from "inferno";
/** @jsx h */ /** @jsxFrag */ import { h } from "preact";
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
    return /*#__PURE__*/ newFragment(260, [
        /*#__PURE__*/ newVNode(17, "div", null, null, {
            dangerouslySetInnerHTML: {
                __html: props.data.markup.content
            }
        })
    ]);
}
