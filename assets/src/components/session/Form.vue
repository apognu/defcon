<template lang="pug">
#app.uk-container
  h2.uk-margin-top.uk-text-center.uk-text-light Defcon

  .uk-card.uk-card-default.uk-card-body.uk-margin-top
    h3.uk-card-title Authentication

    p.uk-alert-danger(uk-alert, v-if='message') {{ message }}

    div.uk-margin-bottom
      label.uk-form-label Email address
      .uk-form-controls
        input.uk-input(
          ref='email'
          type='text',
          v-model='email',
          @keyup.enter='authenticate()'
          :disabled='disabled'
        )

    div
      label.uk-form-label Password
      .uk-form-controls
        input.uk-input(
          type='password',
          v-model='password',
          @keyup.enter='authenticate()'
          :disabled='disabled'
        )

    .uk-margin-top
      button.uk-button.uk-button-primary.uk-button-small(@click='authenticate', :disabled='disabled') Sign in
</template>

<script>
import axios from 'axios';

export default {
  inject: ['store', '$http'],

  data: () => ({
    message: null,
    email: '',
    password: '',
    disabled: false,
  }),

  async mounted() {
    this.$refs.email.focus();
  },

  methods: {
    authenticate() {
      this.disabled = true;

      const body = {
        email: this.email,
        password: this.password,
      };

      axios.post('/api/-/token', body)
        .then((response) => {
          this.store.setCredentials(this.email, this.password);
          this.store.setToken(response.data.access_token, response.data.refresh_token);
        }).catch((e) => {
          this.message = `${e.message}: ${e.response.data.message}`;
          this.password = '';
          this.disabled = false;
        });
    },
  },
};
</script>
