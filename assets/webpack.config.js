const path = require('path');
const glob = require('glob');
const sass = require('sass');

const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const TerserPlugin = require('terser-webpack-plugin');
const OptimizeCSSAssetsPlugin = require('optimize-css-assets-webpack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const PurgecssPlugin = require('purgecss-webpack-plugin');

const { VueLoaderPlugin } = require('vue-loader');

const config = {
  optimization: {
    minimizer: [
      new TerserPlugin({ cache: true, parallel: true, sourceMap: false }),
      new OptimizeCSSAssetsPlugin({}),
    ],
  },

  entry: {
    app: glob.sync('./vendor/**/*.js').concat('@babel/polyfill').concat(['./js/app.js']),
  },

  output: {
    filename: 'js/[name].[contenthash].js',
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
            name: '[name].[contenthash].[ext]',
            outputPath: 'static/',
            publicPath: '/assets/static/',
          },
        },
      },

      {
        test: /\.(js|vue)$/,
        exclude: /node_modules/,
        enforce: 'pre',
        loader: 'eslint-loader',
      },
    ],
  },

  plugins: [
    new HtmlWebpackPlugin({
      template: './index.pug',
      filename: 'index.html',
      inject: true,
    }),

    new MiniCssExtractPlugin({ filename: 'css/[name].[contenthash].css' }),

    new PurgecssPlugin({
      paths: glob.sync(`${__dirname}/js/**/*`, { nodir: true }),
      extensions: ['.vue'],
    }),

    new CopyWebpackPlugin([{
      from: 'static/',
      to: 'static',
    }]),

    new VueLoaderPlugin(),
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
