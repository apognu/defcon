<template lang="pug">
div
  h2 Incidents

  div(v-if='outages')
    .uk-alert-success.uk-margin.uk-padding.uk-border-rounded(
      v-if='outages.length === 0'
    )
      h3.uk-margin-remove.uk-text-center
        span.uk-margin-right(uk-icon='icon: check; ratio: 2')
        | Everything is fine

    .uk-alert-danger.uk-margin.uk-padding.uk-border-rounded(v-else)
      h3.uk-margin-remove.uk-text-center
        span.uk-margin-right(uk-icon='icon: warning; ratio: 2')
        | {{ outages.length }} active incident(s)

    .uk-card.uk-card-default.uk-card-body(v-if='outages.length > 0')
      table.uk-table.uk-table-middle
        tbody
          tr(
            v-for='outage in outages',
            is='OutageRow',
            :outage='outage',
            :key='outage.uuid'
          )
</template>

<script>
import axios from 'axios';

import OutageRow from '@/components/outages/Row.vue';

export default {
  components: {
    OutageRow,
  },

  data: () => ({
    refresher: undefined,
    outages: undefined,
  }),

  async mounted() {
    axios.get('/api/outages').then((response) => {
      this.outages = response.data;
    });

    this.refresher = setInterval(this.refresh, 5000);
  },

  destroyed() {
    clearInterval(this.refresher);
  },

  methods: {
    refresh() {
      axios.get('/api/outages').then((response) => {
        this.outages = response.data;
      });
    },
  },
};
</script>

<style lang="scss" scoped>
</style>
