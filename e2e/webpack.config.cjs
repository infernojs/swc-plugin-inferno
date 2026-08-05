const path = require("path");

// Absolute path to the locally built plugin. `run.mjs` builds it before
// invoking webpack, and asserts it exists.
const pluginPath = path.resolve(
  __dirname,
  "..",
  "target",
  "wasm32-wasip1",
  "release",
  "swc_plugin_inferno.wasm",
);

module.exports = {
  mode: "production",
  entry: path.resolve(__dirname, "fixture", "index.jsx"),
  devtool: false,
  optimization: {
    // Keep the emitted `createVNode(...)` calls readable so the assertions in
    // run.mjs check the plugin output rather than the minifier's output.
    minimize: false,
  },
  module: {
    rules: [
      {
        test: /\.jsx$/,
        exclude: /node_modules/,
        use: {
          loader: "swc-loader",
          options: {
            jsc: {
              parser: { syntax: "ecmascript", jsx: true },
              experimental: {
                plugins: [[pluginPath, {}]],
              },
              target: "es2022",
            },
          },
        },
      },
    ],
  },
  resolve: {
    extensions: [".jsx", ".js"],
  },
  output: {
    filename: "bundle.js",
    path: path.resolve(__dirname, "dist"),
    clean: true,
  },
  stats: "errors-warnings",
};
