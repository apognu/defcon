import Vue from 'vue';
import UIkit from 'uikit';

const helpers = {
  error: (message) => {
    UIkit.notification(message);
  },

  datetime: (dt) => Vue.prototype.$moment(dt).format('MMMM Do, YYYY [at] HH:mm:ss ZZ'),
};

export default {
  install: () => {
    Vue.prototype.$helpers = helpers;
    Vue.$helpers = helpers;
  },
};
