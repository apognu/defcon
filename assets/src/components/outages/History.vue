<template lang="pug">
div
  h2 Incident history

  div(v-if='outages && outages.length > 0')
    .uk-card.uk-card-default.uk-card-body(v-if='outages.length > 0')
      table.uk-table.uk-table-middle
        tbody
          tr(v-for='outage in outages', is='vue:OutageRow', :outage='outage', :key='outage.uuid')

  .uk-placeholder(v-else) There are no incidents in recorded history.
</template>

<script>
import OutageRow from '~/components/outages/Row.vue';

export default {
  components: {
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
      const from = this.$moment().subtract(30, 'day').format('YYYY-MM-DD');
      const to = this.$moment().format('YYYY-MM-DD');

      this.$http().get(`/api/outages?from=${from}&to=${to}&limit=20`).then((response) => {
        this.outages = response.data;
        this.outages.sort((left, right) => {
          if (left.ended_on === undefined) {
            return false;
          }
          if (right.ended_on === undefined) {
            return true;
          }

          return left.started_on < right.started_on;
        });
      });
    },
  },
};
</script>
