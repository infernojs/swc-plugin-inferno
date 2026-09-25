#!/usr/bin/env node
// End-to-end test for the published artifact shape: bundle real JSX through
// webpack + swc-loader using the locally built `swc_plugin_inferno.wasm`.
//
// This exists because unit tests (`cargo test`) exercise the transform as a
// native rlib and therefore cannot catch failures that only appear in the
// wasm32-wasip1 artifact. Versions 2.16.0 - 2.16.2 passed every `cargo test`
// yet hung forever inside the host<->guest bridge for every consumer, before
// the transform function was even entered. The hard timeout below is the guard
// against that class of regression: a hang must fail the build, not wait.

import { spawn, spawnSync } from "node:child_process";
import { existsSync, readFileSync, rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { transformSync } from "@swc/core";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..");
const wasmPath = resolve(repoRoot, "target/wasm32-wasip1/release/swc_plugin_inferno.wasm");
const bundlePath = resolve(here, "dist/bundle.js");

const TIMEOUT_MS = Number(process.env.E2E_TIMEOUT_MS ?? 120_000);

function run(cmd, args, opts = {}) {
  return new Promise((resolvePromise) => {
    const child = spawn(cmd, args, { stdio: "inherit", ...opts });
    let timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill("SIGKILL");
    }, opts.timeoutMs ?? TIMEOUT_MS);

    child.on("error", (err) => {
      clearTimeout(timer);
      resolvePromise({ code: null, timedOut: false, error: err });
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      resolvePromise({ code, timedOut, error: null });
    });
  });
}

function fail(message) {
  console.error(`\n[e2e] FAIL: ${message}\n`);
  process.exit(1);
}

if (process.env.E2E_SKIP_BUILD !== "1") {
  console.error("[e2e] building wasm plugin (cargo build --release --target wasm32-wasip1)");
  const build = await run("cargo", ["build", "--target", "wasm32-wasip1", "--release"], {
    cwd: repoRoot,
    timeoutMs: Number(process.env.E2E_BUILD_TIMEOUT_MS ?? 1_800_000),
  });
  if (build.error) fail(`could not run cargo: ${build.error.message}`);
  if (build.timedOut) fail("cargo build timed out");
  if (build.code !== 0) fail(`cargo build exited with code ${build.code}`);
}

if (!existsSync(wasmPath)) {
  fail(`plugin wasm not found at ${wasmPath} - run 'npm run build' first`);
}

rmSync(resolve(here, "dist"), { recursive: true, force: true });

console.error(`[e2e] bundling fixture with webpack (timeout ${TIMEOUT_MS}ms)`);
const webpack = await run(
  process.execPath,
  [resolve(here, "node_modules/webpack-cli/bin/cli.js"), "--config", resolve(here, "webpack.config.cjs")],
  { cwd: here },
);

if (webpack.error) fail(`could not run webpack: ${webpack.error.message}`);
if (webpack.timedOut) {
  fail(
    `webpack did not finish within ${TIMEOUT_MS}ms.\n` +
      `        This is the signature of a miscompiled wasm plugin: @swc/core spins inside\n` +
      `        the plugin runner and never returns. Check the Rust toolchain used to build\n` +
      `        the artifact (see rust-toolchain.toml) before looking at the transform code.`,
  );
}
if (webpack.code !== 0) fail(`webpack exited with code ${webpack.code}`);

if (!existsSync(bundlePath)) fail(`webpack produced no bundle at ${bundlePath}`);

const bundle = readFileSync(bundlePath, "utf8");

// Assert against the fixture's own section of the bundle. inferno's runtime is
// concatenated into the same file and defines `createVNode` & friends, so
// searching the whole bundle would pass even if nothing had been transformed.
const marker = "./fixture/index.jsx";
const markerAt = bundle.indexOf(marker);
if (markerAt === -1) fail(`bundle does not contain the fixture module (${marker})`);
const transformed = bundle.slice(markerAt);

const expectations = [
  // <li className="item">{label}</li> - className hoisted to its own argument,
  // single expression child.
  [/createVNode\(1, "li", "item", label,/, "element vnode with hoisted className"],
  // <Item key={item} label={item} /> - component vnode, key as trailing arg.
  [/createComponentVNode\(2, Item, \{\s*label: item\s*\}, item\)/, "component vnode with key"],
  // <ul $HasKeyedChildren> - childFlags 8 (HasKeyedChildren) from the directive.
  [/createVNode\(1, "ul", null, items\.map\([\s\S]*?\), 8\)/, "$HasKeyedChildren directive"],
  // <>...</> - fragment.
  [/createFragment\(\[/, "fragment vnode"],
  // pure annotations survive into the bundle.
  [/\/\*#__PURE__\*\/ ?createVNode\(1, "div", "app",/, "#__PURE__ annotation"],
];

const failures = expectations.filter(([re]) => !re.test(transformed)).map(([, what]) => what);
if (failures.length > 0) {
  fail(`transformed fixture is missing: ${failures.join(", ")}`);
}

// The plugin must consume every JSX node; nothing JSX-shaped may survive.
if (/<\/(div|ul|li|h1|span)>/.test(transformed)) {
  fail("bundle still contains untransformed JSX");
}

// Plugin options are parsed inside the wasm guest, so only this test covers them.
const optionsFixture = `
import { normalizeProps, forwardRef } from "inferno";
export function normalize(vNode) {
  normalizeProps(vNode);
  return vNode;
}
export const Comp = forwardRef((props) => <div {...props} />);
`;

function transformWith(pluginOptions) {
  return transformSync(optionsFixture, {
    filename: "options.jsx",
    jsc: {
      parser: { syntax: "ecmascript", jsx: true },
      experimental: { plugins: [[wasmPath, pluginOptions]] },
      target: "es2022",
    },
  }).code;
}

const withDefaults = transformWith({});
if (!/\/\*#__PURE__\*\/ ?forwardRef\(/.test(withDefaults)) {
  fail(`default options: forwardRef(...) is not annotated #__PURE__:\n${withDefaults}`);
}
// Both the normalizeProps call of a spread element and the vNode it wraps are annotated, so that
// a minifier can drop an unused element completely.
if (!/\/\*#__PURE__\*\/ ?normalizeProps\(\/\*#__PURE__\*\/ ?createVNode\(1, "div"/.test(withDefaults)) {
  fail(`default options: the vNode inside normalizeProps(...) is not annotated #__PURE__:\n${withDefaults}`);
}
// normalizeProps mutates its argument, so a minifier must never drop this call.
if (/#__PURE__\*\/ ?normalizeProps\(vNode\)/.test(withDefaults)) {
  fail(`normalizeProps(vNode) must not be annotated #__PURE__:\n${withDefaults}`);
}

const withPureFalse = transformWith({ pure: false });
if (withPureFalse.includes("#__PURE__")) {
  fail(`{ pure: false }: output still contains #__PURE__:\n${withPureFalse}`);
}

// Invalid options must fail the transform with a clear error. The plugin
// reports them as a diagnostic, which @swc/core turns into a thrown error.
let invalidError = null;
try {
  transformWith({ bogus: 1 });
} catch (err) {
  invalidError = err;
}
if (invalidError === null) fail("invalid plugin options were accepted");
if (!String(invalidError.message ?? invalidError).includes("swc-plugin-inferno: invalid plugin options")) {
  fail(`invalid plugin options did not report a clear error:\n${invalidError.message ?? invalidError}`);
}

// A `null` config means the defaults.
if (!/\/\*#__PURE__\*\/ ?forwardRef\(/.test(transformWith(null))) {
  fail("null plugin options: defaults were not applied");
}

// swc drops the warnings of a transform that succeeds, so the plugin prints its useless flag
// warnings itself. They go to the stderr of the process, which only a child process can read.
const uselessFlagSource = "<div $HasVNodeChildren><h1>Hi</h1></div>;";
const uselessFlagMessage =
  "$HasVNodeChildren is not needed: the children are known at compile time, so the plugin sets their child flags.";

function transformUselessFlag(pluginOptions) {
  return transformSync(uselessFlagSource, {
    filename: "flags.jsx",
    jsc: {
      parser: { syntax: "ecmascript", jsx: true },
      experimental: { plugins: [[wasmPath, pluginOptions]] },
    },
  }).code;
}

function uselessFlagWarnings(pluginOptions) {
  const script = `
    const { transformSync } = require("@swc/core");
    transformSync(${JSON.stringify(uselessFlagSource)}, {
      filename: "flags.jsx",
      jsc: {
        parser: { syntax: "ecmascript", jsx: true },
        experimental: { plugins: [[${JSON.stringify(wasmPath)}, ${JSON.stringify(pluginOptions)}]] },
      },
    });
  `;
  const child = spawnSync(process.execPath, ["-e", script], { cwd: here, encoding: "utf8", timeout: TIMEOUT_MS });

  if (child.status !== 0) fail(`useless flag transform failed with ${JSON.stringify(pluginOptions)}:\n${child.stderr}`);
  return child.stderr;
}

const expectedWarning =
  `swc-plugin-inferno: flags.jsx:1:6: ${uselessFlagMessage}`;
const printedWarning = uselessFlagWarnings({});
if (!printedWarning.includes(expectedWarning) || !printedWarning.includes("    |      ^^^^^^^^^^^^^^^^^")) {
  fail(`useless flag: the warning with its code frame was not printed:\n${printedWarning}`);
}
if (uselessFlagWarnings({ uselessFlags: "off" }).includes("swc-plugin-inferno")) {
  fail('{ uselessFlags: "off" }: a warning was printed');
}

let uselessFlagError = null;
try {
  transformUselessFlag({ uselessFlags: "error" });
} catch (err) {
  uselessFlagError = err;
}
if (uselessFlagError === null) fail('{ uselessFlags: "error" }: the useless flag was accepted');
if (!String(uselessFlagError.message ?? uselessFlagError).includes(uselessFlagMessage)) {
  fail(`{ uselessFlags: "error" }: the error does not explain the flag:\n${uselessFlagError.message ?? uselessFlagError}`);
}

let levelError = null;
try {
  transformUselessFlag({ uselessFlags: "warning" });
} catch (err) {
  levelError = err;
}
if (levelError === null) fail('{ uselessFlags: "warning" }: the unknown level was accepted');
if (!String(levelError.message ?? levelError).includes('the uselessFlags option must be "warn", "error" or "off", got "warning".')) {
  fail(`{ uselessFlags: "warning" }: the unknown level did not report a clear error:\n${levelError.message ?? levelError}`);
}

// Fast refresh reads the host's source map and the module scope that the host's resolver set,
// which only the wasm guest sees.
const refreshed = transformSync(
  `
import { useState } from "inferno-hooks";
import { useFancyState } from "./hooks";
export function App() {
  const [count, setCount] = useState(0);
  const [value] = useFancyState();
  return <h1>{count}{value}</h1>;
}
export const Card = Inferno.memo(() => <div />);
`,
  {
    filename: "refresh.jsx",
    jsc: {
      parser: { syntax: "ecmascript", jsx: true },
      experimental: { plugins: [[wasmPath, { development: true, refresh: { emitFullSignatures: true } }]] },
      target: "es2022",
    },
  },
).code;
const refreshExpectations = [
  // Hook keys include the source text of the bindings and of useState's initial value.
  [/"useState\{\[count, setCount\]\(0\)\}\\nuseFancyState\{\[value\]\}"/, "hook signature with source text"],
  // An imported custom hook is in scope: no forced reset, listed in the hooks array.
  [/, false, function\(\) \{\s*return \[\s*useFancyState\s*\];/, "imported custom hook in scope"],
  // A member expression HOC is named after its source text.
  [/\$RefreshReg\$\(_c\d*, "Card\$Inferno\.memo"\)/, "HOC registration name"],
];
const refreshFailures = refreshExpectations.filter(([re]) => !re.test(refreshed)).map(([, what]) => what);
if (refreshFailures.length > 0) {
  fail(`fast refresh output is missing: ${refreshFailures.join(", ")}\n${refreshed}`);
}

console.error(`[e2e] OK - ${bundle.length} byte bundle, all assertions passed`);
