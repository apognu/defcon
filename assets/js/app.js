import '../css/app.scss';

import UIkit from 'uikit';
import Icons from 'uikit/dist/js/uikit-icons';

import Vue from 'vue';
import Moment from 'moment';
import VueMoment from 'vue-moment';
import { extendMoment } from 'moment-range';

import router from '@/router';
import App from '@/components/App.vue';
import Filters from '@/components/filters';

UIkit.use(Icons);

Vue.config.productionTip = false;

const moment = extendMoment(Moment);

Vue.use(VueMoment, { moment });

const _app = new Vue({
  el: '#app',
  router,
  components: { App, Filters },
});
