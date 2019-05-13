const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

const appConfig = {
  entry: {
      app: "./bootstrap.js",
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].js",
    chunkFilename: "app.[name].js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin(['index.html'])
  ],
};

const workerConfig = {
  entry: {
      worker: "./bootstrap_worker.js",
  },
  target: "webworker",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].js",
    chunkFilename: "worker.[name].js",
  },
  mode: "development",
};

module.exports = [appConfig, workerConfig];
