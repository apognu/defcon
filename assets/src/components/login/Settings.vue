<template lang="pug">
.settings
  h2.uk-margin-remove-top Settings

  .heading.uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin-bottom
    h3.uk-card-title Change password

    p.uk-alert-success(uk-alert, v-if='alerts.success') {{ alerts.success }}
    p.uk-alert-danger(uk-alert, v-if='alerts.error') {{ alerts.error }}

    div.uk-margin-bottom
      label.uk-form-label Current password
      .uk-form-controls
        input.uk-input(
          type='password',
          v-model='password',
          @keyup.enter='submit()'
          :disabled='disabled'
        )

    div.uk-margin-bottom
      label.uk-form-label New password
      .uk-form-controls
        input.uk-input(
          type='password',
          v-model='new_password',
          @keyup.enter='submit()'
          :disabled='disabled'
        )

    div.uk-margin-bottom
      label.uk-form-label Confirm your new password
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
    disabled: false,
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

  methods: {
    submit() {
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
        })
        .catch((e) => {
          this.alerts.error = `There was an error updating your password: ${e.response.data.message}.`;
        });
    },
  },
};
</script>
