<template lang="pug">
div(v-if='status')
  .uk-alert-success.uk-margin.uk-padding.uk-border-rounded(v-if='status.ok')
    h3.uk-margin-remove.uk-text-center
      span.uk-margin-right(uk-icon='icon: check; ratio: 2')
      | Everything is fine

  .uk-alert-danger.uk-margin.uk-padding.uk-border-rounded(v-else)
    h3.uk-margin-remove.uk-text-center
      span.uk-margin-right(uk-icon='icon: warning; ratio: 2')
      | {{ status.outages.global }} active incident(s)
</template>

<script>
import axios from 'axios';

export default {
  data: () => ({
    refresher: undefined,
    status: undefined,
  }),

  async mounted() {
    this.refresh();

    this.refresher = setInterval(this.refresh, 5000);
  },

  destroyed() {
    clearInterval(this.refresh);
  },

  methods: {
    refresh() {
      axios.get('/api/status').then((response) => {
        this.status = response.data;
      });
    },
  },
};
</script>
