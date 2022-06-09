const { defineConfig } = require('@vue/cli-service');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const path = require('path');

module.exports = defineConfig({
  transpileDependencies: true,
  outputDir: path.resolve(__dirname, '../dist/'),
  assetsDir: '',
  publicPath: '/assets',
  configureWebpack: {
    plugins: [
      new CopyWebpackPlugin({
        patterns: [
          { from: 'static/', to: '' },
        ],
      }),
    ],

    resolve: {
      alias: {
        '~': path.resolve(__dirname, 'src/'),
      },
    },
  },
});
