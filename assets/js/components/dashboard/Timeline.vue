<template lang="pug">
.uk-card.uk-card-default.uk-card-body.uk-margin
  h3 {{ title }}

  .timeline(v-if='statistics')
    div(v-for='day in days')
      .bar.error.uk-margin-small-right.uk-margin-small-bottom(
        v-if='day.format("YYYY-MM-DD") in statistics',
        :uk-tooltip='`title: ${day.format("YYYY-MM-DD")}`'
      )
      .bar.success.uk-margin-small-right.uk-margin-small-bottom(
        v-else,
        :uk-tooltip='`title: ${day.format("YYYY-MM-DD")}`'
      )
</template>

<script>
import axios from 'axios';

export default {
  props: {
    title: {
      type: String,
      default: 'Timeline',
    },

    check: {
      type: String,
      default: null,
    },

    period: {
      type: Number,
      default: 30,
    },
  },

  data: () => ({
    refresher: undefined,
    days: [],
    statistics: undefined,
  }),

  async mounted() {
    this.refresh();

    this.refresher = setInterval(this.refresh, 5000);
  },

  destroyed() {
    clearInterval(this.refresher);
  },

  methods: {
    refresh() {
      const start = this.$moment().subtract(this.period - 1, 'day');
      const end = this.$moment();
      const range = this.$moment.range(start, end);

      this.days = Array.from(range.by('day'));

      axios.get('/api/status').then((response) => {
        this.status = response.data;
      });

      const from = start.format('YYYY-MM-DD');
      const to = end.format('YYYY-MM-DD');

      let check = '';

      if (this.check !== null) {
        check = `check=${this.check}&`;
      }

      axios
        .get(`/api/statistics?${check}&from=${from}&to=${to}`)
        .then((response) => {
          this.statistics = response.data;
        });
    },
  },
};
</script>

<style lang="scss" scoped>
.timeline {
  display: grid;
  grid-template-columns: repeat(30, 1fr);

  .bar {
    height: 32px;
    margin: auto;
    border-radius: 8px;

    &.success {
      background: #1abc9c;
    }

    &.error {
      background: #c0392b;
    }
  }
}
</style>
