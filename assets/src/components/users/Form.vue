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
        Field(:model='v$.user.name')
          input.uk-input(
            type='text',
            v-model='v$.user.name.$model',
            @keyup.enter='save()'
          )

  .uk-form-horizontal(v-if='user || new_record')
    .uk-margin
      label.uk-form-label Username / e-mail address
      .uk-form-controls
        Field(:model='v$.user.email')
          input.uk-input(
            type='text',
            v-model='v$.user.email.$model',
            @keyup.enter='save()'
          )

  .uk-form-horizontal(v-if='user || new_record')
    .uk-margin
      label.uk-form-label Password
      .uk-form-controls
        Field(:model='v$.user.password')
          input.uk-input(
            type='password',
            v-model='v$.user.password.$model',
            @keyup.enter='save()'
          )

  .uk-form-horizontal(v-if='user || new_record')
    .uk-margin
      label.uk-form-label Confirm password
      .uk-form-controls
        Field(:model='v$.password_confirmation')
          input.uk-input(
            type='password',
            v-model='v$.password_confirmation.$model',
            @keyup.enter='save()'
          )

    .uk-margin-top
      button.uk-button.uk-button-primary.uk-button-small(@click='save', :disabled='v$.$invalid') Save user
</template>

<script>
import { useVuelidate } from '@vuelidate/core';
import {
  required,
  email,
  sameAs,
  requiredIf,
} from '@vuelidate/validators';

import Field from '~/components/partials/Field.vue';

export default {
  inject: ['store', '$http', '$helpers'],

  setup: () => ({
    v$: useVuelidate(),
  }),

  components: {
    Field,
  },

  data: () => ({
    user: {},
    password_confirmation: undefined,
  }),

  validations() {
    return {
      user: {
        name: { required },
        email: { required, email },
        password: {
          required: requiredIf(this.new_record),
        },
      },
      password_confirmation: {
        required: requiredIf(this.password),
        sameAs: sameAs(this.user.password),
      },
    };
  },

  computed: {
    new_record() {
      return this.$route.meta.action === 'new';
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
      this.v$.$validate();

      if (!this.v$.$error) {
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
            });
        } else {
          this.$http()
            .patch(`/api/users/${this.$route.params.uuid}`, body)
            .then(() => {
              if (this.user.uuid === this.store.identity.uuid) {
                this.$http().get('/api/-/me').then((response) => {
                  this.store.setIdentity(response.data);

                  this.$router.push({ name: 'users' });
                });
              } else {
                this.$router.push({ name: 'users' });
              }
            });
        }
      }
    },
  },
};
</script>
