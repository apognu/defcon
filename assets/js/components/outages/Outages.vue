<template lang="pug">
div
  h2 Incidents

  Status

  div(v-if='outages')
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

import Status from '@/components/dashboard/Status.vue';
import OutageRow from '@/components/outages/Row.vue';

export default {
  components: {
    Status,
    OutageRow,
  },

  data: () => ({
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
