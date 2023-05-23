const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyPlugin = require("copy-webpack-plugin");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");

const distPath = path.resolve(__dirname, "dist");
const staticFilesSrc = path.resolve(__dirname, "static");
const audioFilesSrc = path.resolve(__dirname, "../../assets");

module.exports = (env, argv) => {
  const isProduction = argv.mode === "production";
  return {
    devServer: {
      port: isProduction ? 8000 : 4000,
      static: {
        directory: distPath,
      },
      historyApiFallback: {
        index: "/",
      },
      open: true,
    },
    experiments: {
      syncWebAssembly: true,
    },
    entry: "./index.js",
    output: {
      path: distPath,
      filename: "main.js",
    },
    module: {
      rules: [
        {
          test: /\.s[ac]ss$/i,
          use: ["style-loader", "css-loader", "sass-loader"],
        },
      ],
    },
    plugins: [
      new CopyPlugin({
        patterns: [
          { from: staticFilesSrc, to: distPath },
          { from: audioFilesSrc, to: distPath },
        ],
      }),
      new WasmPackPlugin({
        crateDirectory: __dirname,
        forceMode: isProduction ? "production" : "development",
      }),
      new CleanWebpackPlugin(),
    ],
    mode: isProduction ? "production" : "development",
    // ignore for now
    // todo - consider asset size limitations
    performance: {
      maxEntrypointSize: Infinity,
      maxAssetSize: Infinity,
    },
  };
};
