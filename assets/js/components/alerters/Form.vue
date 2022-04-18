<template lang="pug">
div
  h2(v-if='new_record') Create an alerter
  h2(v-else) Edit an alerter

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
          option(value='webhook') Plain webhook
          option(value='slack') Slack incoming webhook

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
import axios from 'axios';

export default {
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
      axios.get(`/api/alerters/${this.$route.params.uuid}`).then((response) => {
        this.alerter = response.data;
      });
    }
  },

  methods: {
    save() {
      const body = this.alerter;

      delete body.uuid;

      if (this.new_record) {
        axios.post('/api/alerters', body).then(() => {
          this.$router.push({ name: 'alerters' });
        });
      } else {
        axios.put(`/api/alerters/${this.$route.params.uuid}`, body).then(() => {
          this.$router.push({ name: 'alerters' });
        });
      }
    },
  },
};
</script>
