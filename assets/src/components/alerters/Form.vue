<template lang="pug">
div(v-if='alerter')
  h2(v-if='new_record') New alerter
  template(v-else-if='alerter')
    p.uk-margin-remove.uk-text-small.uk-text-bolder.uk-text-uppercase Edit alerter
    h2.uk-margin-remove-top {{ alerter.name }}

  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Alerter name
      .uk-form-controls
        Field(:model='v$.alerter.name')
          input.uk-input(
            type='text',
            v-model='v$.alerter.name.$model',
            @keyup.enter='save()'
          )

    .uk-margin
      label.uk-form-label Alerter type
      .uk-form-controls
        select.uk-select(v-model='alerter.kind')
          option(value='webhook') {{ $filters.alerterkind("webhook") }}
          option(value='slack') {{ $filters.alerterkind("slack") }}
          option(value='pagerduty') {{ $filters.alerterkind("pagerduty") }}

    .uk-margin(v-if='field_shown.url')
      label.uk-form-label {{ $filters.alerterlabels(alerter.kind).url }}
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='alerter.url',
          @keyup.enter='save()'
        )

    .uk-margin(v-if='field_shown.username')
      label.uk-form-label {{ $filters.alerterlabels(alerter.kind).username }}
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='alerter.username',
          @keyup.enter='save()'
        )

    .uk-margin(v-if='field_shown.password')
      label.uk-form-label {{ $filters.alerterlabels(alerter.kind).password }}
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='alerter.password',
          @keyup.enter='save()'
        )

    .uk-margin
      button.uk-button.uk-button-primary.uk-button-small(@click='save') Save alerter
</template>

<script>
import { useVuelidate } from '@vuelidate/core';
import { required } from '@vuelidate/validators';

import Field from '~/components/partials/Field.vue';

export default {
  inject: ['$http', '$filters'],

  setup: () => ({
    v$: useVuelidate(),
  }),

  components: {
    Field,
  },

  data: () => ({
    alerter: undefined,
  }),

  validations: () => ({
    alerter: {
      name: { required },
      url: { required },
    },
  }),

  computed: {
    new_record() {
      return this.$route.meta.action === 'new';
    },

    field_shown() {
      const fields = { url: false, username: false, password: false };

      switch (this.alerter.kind) {
        case 'webhook':
        case 'slack':
          fields.url = true;
          break;
        case 'pagerduty':
          fields.password = true;
          break;
        default:
          return {};
      }

      return fields;
    },
  },

  async mounted() {
    if (this.new_record) {
      this.alerter = {};
    } else {
      this.$http().get(`/api/alerters/${this.$route.params.uuid}`).then((response) => {
        this.alerter = response.data;
      });
    }
  },

  methods: {
    save() {
      this.v$.$validate();

      if (!this.v$.$error()) {
        const body = this.alerter;

        delete body.uuid;

        if (this.new_record) {
          this.$http()
            .post('/api/alerters', body)
            .then(() => {
              this.$router.push({ name: 'alerters' });
            });
        } else {
          this.$http()
            .put(`/api/alerters/${this.$route.params.uuid}`, body)
            .then(() => {
              this.$router.push({ name: 'alerters' });
            });
        }
      }
    },
  },
};
</script>
