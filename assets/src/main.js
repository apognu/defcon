import '../css/app.scss';

import UIkit from 'uikit';
import Icons from 'uikit/dist/js/uikit-icons';

import { createApp } from 'vue';
import { createPinia } from 'pinia';
import Moment from 'moment';
import { extendMoment } from 'moment-range';

import router from '~/router';
import Dispatch from '~/components/Dispatch.vue';
import Helpers from '~/components/helpers';

UIkit.use(Icons);

const moment = extendMoment(Moment);

const app = createApp(Dispatch);

app.use(createPinia());
app.use(router);

app.config.globalProperties.$moment = moment;

Helpers.setup(app);

app.mount('#app');
