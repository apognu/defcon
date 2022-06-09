module.exports = {
  env: {
    node: true,
  },
  extends: [
    'plugin:vue/vue3-essential',
    '@vue/airbnb',
  ],
  parserOptions: {
    parser: '@babel/eslint-parser',
  },
  rules: {
    'no-unused-vars': [
      'error',
      {
        varsIgnorePattern: '^_.*',
      },
    ],
    'no-param-reassign': [
      'error',
      {
        props: true,
        ignorePropertyModificationsFor: [
          'state',
          'acc',
          'e',
          'ctx',
          'req',
          'request',
          'res',
          'response',
          '$scope',
        ],
      },
    ],
    'no-underscore-dangle': 'off',
    'max-len': [
      'error',
      {
        code: 200,
      },
    ],
    'vue/multi-word-component-names': 'off',
  },
};
