<template lang="pug">
div
  h2 Dashboard

  div(v-if='status')
    .uk-alert-success.uk-margin.uk-padding.uk-border-rounded(v-if='status.ok')
      h3.uk-margin-remove.uk-text-center
        span.uk-margin-right(uk-icon='icon: check; ratio: 2')
        | Everything is fine

    .uk-alert-danger.uk-margin.uk-padding.uk-border-rounded(v-else)
      h3.uk-margin-remove.uk-text-center
        span.uk-margin-right(uk-icon='icon: warning; ratio: 2')
        | {{ status.outages.global }} active incident(s)

  div(v-if='statistics')
    h3 Timeline

    .timeline
      div(v-for='day in days')
        .bubble.error.uk-margin-small-right.uk-margin-small-bottom(
          v-if='day.format("YYYY-MM-DD") in statistics',
          :uk-tooltip='`title: ${day.format("YYYY-MM-DD")}`'
        )
        .bubble.success.uk-margin-small-right.uk-margin-small-bottom(
          v-else,
          :uk-tooltip='`title: ${day.format("YYYY-MM-DD")}`'
        )
</template>

<script>
import axios from 'axios';

export default {
  data: () => ({
    status: undefined,
    statistics: undefined,
    days: [],
  }),

  async mounted() {
    const start = this.$moment().subtract(89, 'day');
    const end = this.$moment();
    const range = this.$moment.range(start, end);

    this.days = Array.from(range.by('day'));

    axios.get('/api/status').then((response) => {
      this.status = response.data;
    });

    const from = start.format('YYYY-MM-DD');
    const to = end.format('YYYY-MM-DD');

    axios.get(`/api/statistics?from=${from}&to=${to}`).then((response) => {
      this.statistics = response.data;
    });
  },
};
</script>

<style lang="scss" scoped>
.timeline {
  display: grid;
  grid-template-columns: repeat(30, 1fr);

  .bubble {
    width: 16px;
    height: 16px;
  }
}
</style>
