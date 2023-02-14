import UIkit from 'uikit';
import axios from 'axios';

import { store } from '~/store';

const publicHttp = () => axios.create();

const http = (noRetry) => {
  const defconStore = store();

  const client = axios.create({
    headers: {
      common: {
        Authorization: `Bearer ${defconStore.accessToken}`,
      },
    },
  });

  client.interceptors.response.use(
    null,
    (error) => {
      if (error.response.status !== 401 && error.response.status >= 400 && error.response.status <= 599) {
        if (error.response.data.details) {
          UIkit.notification(`${error.response.data.details}`, { status: 'danger' });
        }
      }

      return Promise.reject(error);
    },
  );

  if (noRetry !== true) {
    client.interceptors.response.use(
      null,
      (error) => {
        const request = error.config;

        if (error.response.status === 401 && !request._retry) {
          request._retry = true;

          const body = {
            refresh_token: defconStore.refreshToken,
          };

          axios.post('/api/-/refresh', body)
            .then((response) => {
              defconStore.setToken(response.data.access_token, response.data.refresh_token);

              request.headers.Authorization = `Bearer ${defconStore.accessToken}`;

              return axios(request);
            }).catch(() => {
              defconStore.revokeToken();
            });
        }

        return Promise.reject(error);
      },
    );
  }

  return client;
};

const helpers = (app) => ({
  error: (message) => {
    UIkit.notification(message);
  },

  datetime: (dt) => app.config.globalProperties.$moment(dt).format('D MMM YYYY [at] HH:mm ZZ'),
  ago: (dt) => app.config.globalProperties.$moment(dt).fromNow(),
  humanize: (duration) => duration.humanize(),
});

const filters = {
  checkkind: (value) => {
    switch (value) {
      case 'app_store': return 'App Store';
      case 'dns': return 'DNS';
      case 'http': return 'HTTP';
      case 'ping': return 'Ping';
      case 'play_store': return 'Google Play';
      case 'tcp': return 'TCP';
      case 'tls': return 'TLS';
      case 'udp': return 'UDP';
      case 'domain': return 'Domain';
      case 'python': return 'Python script';
      case 'deadmanswitch': return 'Dead Man Switch';
      default: return 'Unknown';
    }
  },

  alerterkind: (value) => {
    switch (value) {
      case 'webhook': return 'Webhook';
      case 'slack': return 'Slack';
      case 'pagerduty': return 'Pagerduty';
      default: return 'Unknown';
    }
  },

  alerterlabels: (kind) => {
    switch (kind) {
      case 'webhook':
      case 'slack':
        return { url: 'Webhook URL' };
      case 'pagerduty':
        return { password: 'Integration key' };
      default:
        return {};
    }
  },

  timeline: (kind) => {
    switch (kind) {
      case 'acknowledgement':
        return { class: 'info', message: 'Incident was acknowledged.' };
      case 'site_outage_started':
        return { class: 'error', message: 'Site-local incident started.' };
      case 'site_outage_resolved':
        return { class: 'success', message: 'Site-local incident resolved.' };
      case 'outage_started':
        return { class: 'error', message: 'Incident started.' };
      case 'outage_resolved':
        return { class: 'success', message: 'Incident resolved.' };
      case 'alert_dispatched':
        return { class: 'info', message: 'Alert was dispatched.' };
      default:
        return { class: 'unknown', message: 'An unknown event has occured.' };
    }
  },
};

export default {
  setup: (app) => {
    app.provide('store', store());
    app.provide('$http', http);
    app.provide('$publicHttp', publicHttp);
    app.provide('$helpers', helpers(app));
    app.provide('$filters', filters);
  },
};
