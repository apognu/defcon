import { defineStore, storeToRefs } from 'pinia';

export const store = defineStore('defcon', {
  state: () => ({
    title: '',

    authenticated: undefined,
    identity: undefined,
    accessToken: window.localStorage.getItem('access_token'),
    refreshToken: window.localStorage.getItem('refresh_token'),

    status: undefined,
    incidents: 0,
    statusPage: false,
  }),

  getters: {
    getTitle: (state) => {
      if (state.title !== undefined) {
        return `Defcon • ${state.title}`;
      }

      return 'Defcon';
    },
  },

  actions: {
    setTitle(title) {
      this.title = title;
    },

    setCredentials(email, password) {
      this.email = email;
      this.password = password;

      window.localStorage.setItem('email', email);
      window.localStorage.setItem('password', password);
    },

    setToken(accessToken, refreshToken) {
      this.authenticated = true;
      this.accessToken = accessToken;
      this.refreshToken = refreshToken;

      window.localStorage.setItem('access_token', accessToken);
      window.localStorage.setItem('refresh_token', refreshToken);
    },

    revokeToken() {
      this.authenticated = false;
      this.identity = undefined;
      this.accessToken = undefined;
      this.refreshToken = undefined;

      window.localStorage.removeItem('email');
      window.localStorage.removeItem('password');
      window.localStorage.removeItem('access_token');
      window.localStorage.removeItem('refresh_token');
    },

    setIdentity(identity) {
      this.identity = identity;
    },

    setStatus(status) {
      this.status = status;
    },

    setIncidentCount(count) {
      this.incidents = count;
    },

    setStatusPage(enabled) {
      this.statusPage = enabled;
    },
  },
});

export default {
  store,
  storeToRefs,
};
