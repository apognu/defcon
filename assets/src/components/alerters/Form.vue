<template lang="pug">
div(v-if='alerter')
  h2(v-if='new_record') New alerter
  template(v-else-if='alerter')
    p.uk-margin-remove.uk-text-small.uk-text-bolder.uk-text-uppercase Edit alerter
    h2.uk-margin-remove-top {{ alerter.name }}

  .uk-form-horizontal(v-if='alerter || new_record')
    .uk-margin
      label.uk-form-label Alerter name
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='alerter.name',
          @keyup.enter='save()'
        )

    .uk-margin
      label.uk-form-label Alerter type
      .uk-form-controls
        select.uk-select(v-model='alerter.kind')
          option(value='webhook') {{ $filters.alerterkind("webhook") }}
          option(value='slack') {{ $filters.alerterkind("slack") }}

    .uk-margin
      label.uk-form-label Webhook URL
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='alerter.webhook',
          @keyup.enter='save()'
        )

    .uk-margin
      button.uk-button.uk-button-primary.uk-button-small(@click='save') Save alerter
</template>

<script>
export default {
  inject: ['$http', '$filters'],

  data: () => ({
    alerter: undefined,
  }),

  computed: {
    new_record() {
      return this.$route.meta.action === 'new';
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
      const body = this.alerter;

      delete body.uuid;

      if (this.new_record) {
        this.$http()
          .post('/api/alerters', body)
          .then(() => {
            this.$router.push({ name: 'alerters' });
          })
          .catch((e) => {
            this.$helpers.error(`${e.message}: ${e.response.data.details}`);
          });
      } else {
        this.$http()
          .put(`/api/alerters/${this.$route.params.uuid}`, body)
          .then(() => {
            this.$router.push({ name: 'alerters' });
          })
          .catch((e) => {
            this.$helpers.error(`${e.message}: ${e.response.data.details}`);
          });
      }
    },
  },
};
</script>
