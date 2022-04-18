module.exports = {
  env: {
    browser: true,
    es6: true,
  },
  extends: [
    'eslint:recommended',
    'plugin:vue/recommended',
    'airbnb-base',
  ],
  globals: {
    Atomics: 'readonly',
    SharedArrayBuffer: 'readonly',
  },
  parserOptions: {
    ecmaVersion: 2018,
    sourceType: 'module',
  },
  plugins: [
    'vue',
  ],
  rules: {
    'no-unused-vars': [
      'error',
      {
        varsIgnorePattern: '^_'
      }
    ],
    'no-param-reassign': [
      'error',
      {
        'props': true,
        'ignorePropertyModificationsFor': [
          'state',
          'acc',
          'e',
          'ctx',
          'req',
          'request',
          'res',
          'response',
          '$scope'
        ]
      }
    ],
    'no-underscore-dangle': 'off',
    'max-len': [
      'error',
      {
        'code': 200,
      },
    ],
  },
  settings: {
    'import/resolver': 'webpack'
  },
  globals: {
    '_': true,
  },
};
