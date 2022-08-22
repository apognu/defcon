<template lang="pug">
.settings
  h2.uk-margin-remove-top Settings

  .heading.uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin-bottom
    h3.uk-card-title Theme

    .uk-form-controls
      label
        input.uk-checkbox.uk-margin-right(type="checkbox", v-model='darkMode', @click="switchTheme")
        | Enable dark theme

  .heading.uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin-bottom
    h3.uk-card-title API key

    p Generating a new API key will invalidate the previous one, please note this API key carefully, it will not be shown again.

    button.uk-button.uk-button-primary.uk-button-small(@click='generateApiKey', :disabled='apiKeyDisabled') Generate a new API key

    div.uk-margin-top.uk-margin-bottom(v-if='apiKey')
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='apiKey',
          readonly='true'
          @keyup.enter='submit()'
        )

  .heading.uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin-bottom
    h3.uk-card-title Change password

    p.uk-alert-success(uk-alert, v-if='alerts.success') {{ alerts.success }}
    p.uk-alert-danger(uk-alert, v-if='alerts.error') {{ alerts.error }}

    div.uk-margin-bottom
      label.uk-display-block.uk-margin-small-bottom.uk-form-label Current password
      .uk-form-controls
        input.uk-input(
          type='password',
          v-model='password',
          @keyup.enter='submit()'
          :disabled='disabled'
        )

    div.uk-margin-bottom
      label.uk-display-block.uk-margin-small-bottom.uk-form-label New password
      .uk-form-controls
        input.uk-input(
          type='password',
          v-model='new_password',
          @keyup.enter='submit()'
          :disabled='disabled'
        )

    div.uk-margin-bottom
      label.uk-display-block.uk-margin-small-bottom.uk-form-label Confirm your new password
      .uk-form-controls
        input.uk-input(
          type='password',
          v-model='new_password_confirmation',
          @keyup.enter='submit()'
          :disabled='disabled'
        )

    .uk-margin-top
      button.uk-button.uk-button-primary.uk-button-small(@click='submit', :disabled='disabled || submitDisabled') Change password
</template>

<script>
export default {
  inject: ['$http'],

  data: () => ({
    alerts: {
      success: undefined,
      error: undefined,
    },
    darkMode: false,
    disabled: false,
    apiKeyDisabled: false,
    apiKey: undefined,
    password: '',
    new_password: '',
    new_password_confirmation: '',
  }),

  computed: {
    submitDisabled() {
      if (this.password === '' || this.new_password === '' || this.new_password_confirmation === '') {
        return true;
      }

      if (this.new_password !== this.new_password_confirmation) {
        return true;
      }

      return false;
    },
  },

  async mounted() {
    this.darkMode = document.documentElement.getAttribute('data-theme') === 'dark';
  },

  methods: {
    switchTheme() {
      if (document.documentElement.getAttribute('data-theme') === 'dark') {
        document.documentElement.setAttribute('data-theme', 'light');
        window.localStorage.setItem('theme', 'light');

        this.darkMode = false;
      } else {
        document.documentElement.setAttribute('data-theme', 'dark');
        window.localStorage.setItem('theme', 'dark');

        this.darkMode = true;
      }
    },

    generateApiKey() {
      this.apiKeyDisabled = true;

      this.$http().post('/api/-/apikey').then((response) => {
        this.apiKey = response.data.api_key;
        this.apiKeyDisabled = false;
      });
    },

    submit() {
      if (this.submitDisabled) {
        return;
      }

      this.disabled = true;

      const body = {
        password: this.password,
        new_password: this.new_password,
      };

      this.$http(true).post('/api/-/password', body)
        .then(() => {
          this.password = '';
          this.new_password = '';
          this.new_password_confirmation = '';

          this.alerts.success = 'Your password was updated successfully.';
          this.disabled = false;
        })
        .catch((e) => {
          this.alerts.error = `There was an error updating your password: ${e.response.data.message}.`;
          this.disabled = false;
        });
    },
  },
};
</script>
