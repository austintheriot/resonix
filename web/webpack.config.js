const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');

const distPath = path.resolve(__dirname, "dist");
const devPath = path.resolve(__dirname, "pkg");
const staticFilesSrc = path.resolve(__dirname, "static");
const audioFilesSrc = path.resolve(__dirname, "../audio");

module.exports = (env, argv) => {
  return {
    devServer: {
      contentBase: argv.mode === 'production' ? distPath : devPath,
      compress: argv.mode === 'production',
      port: 8000,
    },
    entry: './index.js',
    output: {
      path: distPath,
      filename: "main.js",
      webassemblyModuleFilename: "main.wasm",
    },
    module: {
      rules: [
        {
          test: /\.s[ac]ss$/i,
          use: [
            'style-loader',
            'css-loader',
            'sass-loader',
          ],
        },
      ],
    },
    plugins: [
      new CopyWebpackPlugin({
        patterns: [
          { from: staticFilesSrc, to: distPath },
          { from: audioFilesSrc, to: distPath },
          { from: staticFilesSrc, to: devPath },
          { from: audioFilesSrc, to: devPath },
        ],
      }),
      new WasmPackPlugin({
        crateDirectory: ".",
      }),
      new CleanWebpackPlugin(),
    ],
    watch: argv.mode !== 'production'
  };
};
