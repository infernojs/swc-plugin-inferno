<p align="center"><a href="https://infernojs.org/" target="_blank"><img width="500" alt="Inferno" title="Inferno" src="https://raw.githubusercontent.com/infernojs/swc-plugin-inferno/main/swc-plugin-inferno-logo.png"></a></p>
<p align="center">
  <a href="https://www.npmjs.com/package/swc-plugin-inferno"><img src="https://img.shields.io/npm/dm/swc-plugin-inferno.svg" alt="Downloads"></a>
  <a href="https://www.npmjs.com/package/swc-plugin-inferno"><img src="https://img.shields.io/npm/v/swc-plugin-inferno.svg" alt="Version"></a>
  <a href="https://www.npmjs.com/package/swc-plugin-inferno"><img src="https://img.shields.io/npm/l/swc-plugin-inferno.svg" alt="License"></a>
</p>

# InfernoJS SWC Plugin

> Plugin for SWC to enable JSX/TSX for Inferno

This plugin transforms JSX and TSX code in your projects to [Inferno](https://github.com/infernojs/inferno) compatible virtual DOM.
It is recommended to use this plugin for compiling JSX for inferno. It is different to other JSX plugins, because it outputs highly optimized inferno specific `createVNode` calls. This plugin also checks children shape during compilation stage to reduce overhead from runtime application.

The plugin generates the same code as [babel-plugin-inferno](https://github.com/infernojs/babel-plugin-inferno), and its
test suite is ported to this repository (`tests/babel_plugin_inferno`).

## How to install

```bash
npm i --save-dev swc-plugin-inferno
```

## How to use

Add swc-plugin-inferno to `.swcrc` configuration

Enable `jsc.parser.jsx` and set `swc-plugin-inferno` into `jsc.experimental.plugins`
For rest of the settings see: https://swc.rs/docs/configuration/compilation
```json
{
  "jsc": {
    "experimental": {
         "plugins": [
            ["swc-plugin-inferno", {
              "pure": true // Enable or disable /*#__PURE__*/ annotations (default: true)
            }]
        ]
    }
  }
}
```

Unknown plugin options are rejected with an error. The options are:

| Option         | Default     | Description                                                                                  |
|----------------|-------------|----------------------------------------------------------------------------------------------|
| `pure`         | `true`      | Add `/*#__PURE__*/` annotations, see below.                                                  |
| `importSource` | `"inferno"` | The module the helpers are imported from, see [Options](#options).                          |
| `development`  | `false`     | Enables fast refresh together with `refresh`. On its own it changes nothing.                 |
| `refresh`      | off         | Fast refresh: `true`, or `{ "refreshReg": "$RefreshReg$", "refreshSig": "$RefreshSig$", "emitFullSignatures": false }`. Only applies when `development` is `true`. |

With `"pure": true` the plugin adds `/*#__PURE__*/` to the calls it generates from JSX, and also to hand-written calls
of Inferno factories such as `forwardRef`, `createRef`, `createPortal` and `createVNode` imported from `inferno` or from
`importSource`. Minifiers remove these calls when their result is unused, so don't call them only for their side
effects. `normalizeProps` is annotated only when its argument is a freshly created vNode, because it mutates the vNode
passed to it.

### Fast refresh

With `"development": true` and `"refresh": true` the plugin registers components for hot reloading like
`react-refresh/babel`: components are passed to `$RefreshReg$`, and the hooks of a component to a signature created by
`$RefreshSig$`. The functions are renamed with `refreshReg` and `refreshSig`. Your hot reloading runtime has to define
them. Any call of a function named `use` followed by a capital letter counts as a hook, for example `useLoaderData` of
inferno-router. A `// @refresh reset` comment in a file remounts its components on every edit.

To use SWC with Webpack install `swc-loader` and add it to the Webpack configuration

```js
{
  mode: 'development',
  entry: './src/index.js',
  module: {
    rules: [
      {
        test: /\.(js|jsx)$/,
        exclude: /(node_modules|bower_components)/,
        use: {
          // `.swcrc` can be used to configure swc
          loader: 'swc-loader',
        },
      }
    ]
  }
}
```


## Examples

```js

// Render a simple div
Inferno.render(<div></div>, container);

// Render a div with text
Inferno.render(<div>Hello world</div>, container);

// Render a div with a boolean attribute
Inferno.render(<div autoFocus='true' />, container);

```

## Fragments

All the following syntaxes are **reserved** for Inferno's createFragment call

```js
<>
    <div>Foo</div>
    <div>Bar</div>
</>


<Fragment>
    <div>Foo</div>
    <div>Bar</div>
</Fragment>

```

## Special flags

This plugin provides few special compile time flags that can be used to optimize an inferno application.

```js
// ChildFlags:
<div $HasTextChildren /> - Children is rendered as pure text
<div $HasVNodeChildren /> - Children is another vNode (Element or Component)
<div $HasNonKeyedChildren /> - Children is always array without keys
<div $HasKeyedChildren /> - Children is array of vNodes having unique keys
<div $ChildFlag={expression} /> - This attribute is used for defining children shape runtime. See inferno-vnode-flags (ChildFlags) for possible values

// Functional flags
<div $ReCreate /> - This flag tells inferno to always remove and add the node. It can be used to replace key={Math.random()}
<div $Flags={expression} /> - Replaces the vNode flags the plugin computes. See inferno-vnode-flags (VNodeFlags) for possible values
```

## Options

swc-plugin-inferno will automatically import the required methods from inferno library.
There is no need to import inferno in every single JSX file. Only import the inferno specific code required by the application.

example:
```js
import {render} from 'inferno'; // only import 'render'

// The plugin will automatically import 'createVNode'
render(<div>1</div>, document.getElementById('root'));
```

A helper that the file already declares at the top level, for example with `import { createVNode } from 'inferno'`, is
used instead of importing it again. In scripts (files without `import` or `export`) the helpers are read from
`require('inferno')`.

Set `importSource` to import the helpers from another module, like the `imports` option of babel-plugin-inferno:

```json
["swc-plugin-inferno", { "importSource": "inferno-compat" }]
```

### Troubleshoot

You can verify `swc-plugin-inferno` is used by looking at the compiled output.
This plugin does not generate calls to `createElement` or `h`, but instead it uses low level InfernoJS API
`createVNode`, `createComponentVNode`, `createFragment` etc. If you see your JSX being transpiled into `createElement` calls
it is a good indication that your build configuration is not correct.
