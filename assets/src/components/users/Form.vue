<template lang="pug">
div(v-if='user')
  h2(v-if='new_record') New user
  template(v-else-if='user')
    p.uk-margin-remove.uk-text-small.uk-text-bolder.uk-text-uppercase Edit user
    h2.uk-margin-remove-top {{ user.name }}

  .uk-form-horizontal(v-if='user || new_record')
    .uk-margin
      label.uk-form-label Name
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='user.name',
          @keyup.enter='save()'
        )

  .uk-form-horizontal(v-if='user || new_record')
    .uk-margin
      label.uk-form-label Username / e-mail address
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='user.email',
          @keyup.enter='save()'
        )

  .uk-form-horizontal(v-if='user || new_record')
    .uk-margin
      label.uk-form-label Password
      .uk-form-controls
        input.uk-input(
          type='password',
          v-model='user.password',
          @keyup.enter='save()'
        )

  .uk-form-horizontal(v-if='user || new_record')
    .uk-margin
      label.uk-form-label Confirm password
      .uk-form-controls
        input.uk-input(
          type='password',
          v-model='password_confirmation',
          @keyup.enter='save()'
        )

    .uk-margin-top
      button.uk-button.uk-button-primary.uk-button-small(@click='save', :disabled='submitDisabled') Save user
</template>

<script>
export default {
  inject: ['store', '$http', '$helpers'],

  data: () => ({
    user: undefined,
    password_confirmation: undefined,
  }),

  computed: {
    new_record() {
      return this.$route.meta.action === 'new';
    },

    submitDisabled() {
      if (this.new_record) {
        if (this.user.password === undefined || this.user.password === '') {
          return true;
        }
      }

      if (this.user.name === undefined || this.user.email === undefined) {
        return true;
      }
      if (this.user.name === '' || this.user.email === '') {
        return true;
      }
      if (this.user.password !== this.password_confirmation) {
        return true;
      }

      return false;
    },
  },

  async mounted() {
    if (this.new_record) {
      this.user = {};
    } else {
      this.$http().get(`/api/users/${this.$route.params.uuid}`).then((response) => {
        this.user = response.data;
      });
    }
  },

  methods: {
    save() {
      if (this.submitDisabled) {
        return;
      }

      const body = {
        name: this.user.name,
        email: this.user.email,
      };

      if (this.user.password !== undefined && this.user.password !== '') {
        body.password = this.user.password;
      }

      if (this.new_record) {
        this.$http()
          .post('/api/users', body)
          .then(() => {
            this.$router.push({ name: 'users' });
          })
          .catch((e) => {
            this.$helpers.error(`${e.message}: ${e.response.data.details}`);
          });
      } else {
        this.$http()
          .patch(`/api/users/${this.$route.params.uuid}`, body)
          .then(() => {
            if (this.user.uuid == this.store.identity.uuid) {
              this.$http().get('/api/-/me').then((response) => {
                this.store.setIdentity(response.data);

                this.$router.push({ name: 'users' });
              });
            } {
              this.$router.push({ name: 'users' });
            }
          })
          .catch((e) => {
            this.$helpers.error(`${e.message}: ${e.response.data.details}`);
          });
      }
    },
  },
};
</script>
