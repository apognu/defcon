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
            is='vue:OutageRow',
            :outage='outage',
            :key='outage.uuid'
          )
</template>

<script>
import Status from '~/components/dashboard/Status.vue';
import OutageRow from '~/components/outages/Row.vue';

export default {
  components: {
    Status,
    OutageRow,
  },

  inject: ['$http'],

  data: () => ({
    outages: undefined,
  }),

  async mounted() {
    this.refresh();

    this.refresher = setInterval(this.refresh, 5000);
  },

  unmounted() {
    clearInterval(this.refresher);
  },

  methods: {
    refresh() {
      this.$http().get('/api/outages').then((response) => {
        this.outages = response.data;
      });
    },
  },
};
</script>

<style lang="scss" scoped>
</style>
