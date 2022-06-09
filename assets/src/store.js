import { defineStore, storeToRefs } from 'pinia';

export const store = defineStore('defcon', {
  state: () => ({
    title: '',
    identity: undefined,
    email: window.localStorage.getItem('email'),
    password: window.localStorage.getItem('password'),
    accessToken: window.localStorage.getItem('access_token'),
    refreshToken: window.localStorage.getItem('refresh_token'),
    status: undefined,
    incidents: 0,
  }),

  getters: {
    getTitle: (state) => {
      if (state.title !== undefined) {
        /* eslint no-irregular-whitespace: "off" */
        return `Defcon • ${state.title}`;
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
      this.accessToken = accessToken;
      this.refreshToken = refreshToken;

      window.localStorage.setItem('access_token', accessToken);
      window.localStorage.setItem('refresh_token', refreshToken);
    },

    revokeToken() {
      this.accessToken = undefined;
      this.refreshToken = undefined;
      this.identity = undefined;

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
  },
});

export default {
  store,
  storeToRefs,
};
