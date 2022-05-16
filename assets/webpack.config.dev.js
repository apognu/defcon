const path = require('path');
const glob = require('glob');
const sass = require('sass');

const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const ESLintPlugin = require('eslint-webpack-plugin');

const { VueLoaderPlugin } = require('vue-loader');

const config = {
  entry: {
    app: glob.sync('./vendor/**/*.js').concat('@babel/polyfill').concat(['./js/app.js']),
  },

  output: {
    filename: 'js/app.js',
    chunkFilename: 'js/app.js',
    path: path.resolve(__dirname, '../dist'),
    publicPath: '/assets/',
    hashFunction: 'xxhash64',
  },

  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
        },
      },

      {
        test: /\.vue$/,
        use: ['vue-loader'],
      },

      {
        test: /\.pug$/,
        oneOf: [
          {
            resourceQuery: /^\?vue/,
            use: ['pug-plain-loader'],
          },
          { use: ['raw-loader', 'pug-plain-loader'] },
        ],
      },

      {
        test: /\.s?(c|a)ss$/,
        use: [
          { loader: MiniCssExtractPlugin.loader },
          { loader: 'css-loader' },
          {
            loader: 'sass-loader',
            options: {
              implementation: sass,
            },
          },
        ],
      },

      {
        test: /\.(ttf|eot|woff)/,
        use: {
          loader: 'file-loader',
          options: {
            outputPath: 'fonts/',
            publicPath: '/assets/fonts/',
          },
        },
      },

      {
        test: /\.(png|jpg)/,
        use: {
          loader: 'file-loader',
          options: {
            esModule: false,
            name: '[name].[ext]',
            outputPath: 'static/',
            publicPath: '/assets/static/',
          },
        },
      },
    ],
  },

  plugins: [
    new HtmlWebpackPlugin({
      template: './index.pug',
      filename: 'index.html',
      inject: true,
      hash: true,
    }),

    new MiniCssExtractPlugin({ filename: 'css/[name].css' }),

    new CopyWebpackPlugin({
      patterns: [
        {
          from: 'static/',
          to: 'static',
        },
      ],
    }),

    new VueLoaderPlugin(),

    new ESLintPlugin(),
  ],

  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'js'),
      vue$: 'vue/dist/vue.esm.js',
    },
    extensions: ['*', '.js', '.vue', '.json'],
  },
};

module.exports = () => config;
