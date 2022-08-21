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
      case 'whois': return 'Domain';
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
      case 'outage_started':
        return { color: '#e55039', message: 'Incident started.' };
      case 'outage_resolved':
        return { color: '#1abc9c', message: 'Incident resolved.' };
      case 'alert_dispatched':
        return { color: '#2980b9', message: 'Alert was dispatched.' };
      default:
        return { color: '#989898', message: 'An unknown event has occured.' };
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
