<template lang="pug">
.uk-grid-match(uk-grid)
  .uk-width-expand
    .timeline.desktop(v-if='outages', class='uk-visible@m')
      div(v-for='day in days')
        .bar.error.uk-margin-small(v-if='day.format("YYYY-MM-DD") in outages', :uk-tooltip='`title: ${day.format("MMMM Do, YYYY")}`')
        .bar.success.uk-margin-small(v-else, :uk-tooltip='`title: ${day.format("MMMM Do, YYYY")}`')

    .timeline.mobile(v-if='outages', class='uk-hidden@m')
      div(v-for='day in days')
        .bar.error.uk-margin-small-right.uk-margin-small-bottom(v-if='day.format("YYYY-MM-DD") in outages', :uk-tooltip='`title: ${day.format("MMMM Do, YYYY")}`')
        .bar.success.uk-margin-small-right.uk-margin-small-bottom(v-else, :uk-tooltip='`title: ${day.format("MMMM Do, YYYY")}`')

  .uk-width-1-1(v-if='showUptime', class='uk-width-1-4@m')
    .uk-card.uk-card-default.uk-card-body.uk-margin
      h3 Uptime
      p.uk-h1.uk-margin-remove.uk-text-bold.uk-text-center(v-if='uptime') {{ formatUptime(uptime) }} %
      p.uk-h1.uk-margin-remove.uk-text-bold.uk-text-center(v-else) -
</template>

<script>
export default {
  inject: ['$http'],

  props: {
    outages: {
      type: Object,
      default: () => ({}),
      required: true,
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
    this.refresher = setInterval(this.refresh, 30000);
  },

  unmounted() {
    clearInterval(this.refresher);
  },

  methods: {
    refresh() {
      const start = this.$moment().subtract(this.period - 1, 'day');
      const end = this.$moment();
      const range = this.$moment.range(start, end);

      this.days = Array.from(range.by('day'));
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
@import '@/../css/colors.scss';

.timeline {
  &.desktop {
    display: grid;
    grid-template-columns: repeat(30, 1fr);

    .bar {
      margin: 3px;
    }
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
      background: $ok;
    }

    &.error {
      background: $error;
    }
  }
}
</style>
