<template lang="pug">
.uk-grid-match.uk-margin(uk-grid)
  .uk-width-expand
    .uk-card.uk-card-default.uk-card-body.uk-margin
      h3 {{ title }}

      .timeline.desktop(v-if='statistics', class='uk-visible@m')
        div(v-for='day in days')
          .bar.error.uk-margin-small-right.uk-margin-small-bottom(v-if='day.format("YYYY-MM-DD") in statistics', :uk-tooltip='`title: ${day.format("YYYY-MM-DD")}`')
          .bar.success.uk-margin-small-right.uk-margin-small-bottom(v-else, :uk-tooltip='`title: ${day.format("MMMM Do, YYYY")}`')

      .timeline.mobile(v-if='statistics', class='uk-hidden@m')
        div(v-for='day in days')
          .bar.error.uk-margin-small-right.uk-margin-small-bottom(v-if='day.format("YYYY-MM-DD") in statistics', :uk-tooltip='`title: ${day.format("YYYY-MM-DD")}`')
          .bar.success.uk-margin-small-right.uk-margin-small-bottom(v-else, :uk-tooltip='`title: ${day.format("MMMM Do, YYYY")}`')

  .uk-width-1-1(v-if='showUptime', class='uk-width-1-4@m')
    .uk-card.uk-card-default.uk-card-body.uk-margin
      h3 Uptime
      p.uk-h1.uk-margin-remove.uk-text-bold.uk-text-center(v-if='uptime') {{ formatUptime(uptime) }} %
      p.uk-h1.uk-margin-remove.uk-text-bold.uk-text-center(v-else) -
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

    showUptime: {
      type: Boolean,
      default: false,
    },
  },

  data: () => ({
    refresher: undefined,
    days: [],
    statistics: undefined,
    uptime: undefined,
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

      axios.get(`/api/statistics?${check}&from=${from}&to=${to}`).then((response) => {
        this.statistics = response.data;
      });

      axios.get(`/api/outages?${check}&from=${from}&to=${to}`).then((response) => {
        const outages = response.data;

        if (outages.length === 0) {
          this.uptime = 100.0;
        } else {
          const fullRange = this.$moment.rangeFromInterval('day', -30, new Date());
          const full = this.period * 24 * 60 * 60;

          const downs = outages.map((outage) => {
            const started = this.$moment(outage.started_on);
            const ended = this.$moment(outage.ended_on);

            return this.$moment.range(started, ended);
          });

          const up = this.subtractRanges(fullRange, downs).reduce((acc, r) => acc + r.diff('seconds'), 0);

          this.uptime = (up / full) * 100;
        }
      });
    },

    subtractRanges(source, others) {
      return [source].flatMap((s) => others.reduce((remaining, o) => remaining.flatMap((r) => r.subtract(o)), [s]));
    },

    formatUptime(value) {
      const tentative = value.toFixed(1);

      if (tentative === '100.0' && value !== 100) {
        return '99.9';
      }

      return value.toFixed(1);
    },
  },
};
</script>

<style lang="scss" scoped>
.timeline {
  &.desktop {
    display: grid;
    grid-template-columns: repeat(30, 1fr);
  }

  &.mobile {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
  }

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
