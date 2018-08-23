const path = require('path');

module.exports = {
  module: {
    rules: [
      {
        test: /\.rs$/,
        use: [{
          loader: 'wasm-loader'
        }, {
          loader: 'rust-native-wasm-loader',
          options: {
            release: true
          }
        }]
      }
    ]
  }
};
