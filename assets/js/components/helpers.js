import Vue from 'vue';
import UIkit from 'uikit';

const helpers = {
  error: (message) => {
    UIkit.notification(message);
  },
};

export default {
  install: () => {
    Vue.prototype.$helpers = helpers;
    Vue.$helpers = helpers;
  },
};
