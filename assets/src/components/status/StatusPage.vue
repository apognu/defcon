<template lang="pug">
div
  h2 Status page

  div(v-if='enabled && status')

    .uk-alert-success.uk-alert-callout.uk-margin.uk-padding.uk-border-rounded(v-if='status.ok')
      h3.uk-margin-remove.uk-text-center
        span.uk-margin-right(uk-icon='icon: check; ratio: 2')
        | Everything is fine

    .uk-alert-danger.uk-alert-callout.uk-margin.uk-padding.uk-border-rounded(v-else)
      h3.uk-margin-remove.uk-text-center
        span.uk-margin-right(uk-icon='icon: warning; ratio: 2')
        | {{ status.outages }} active incident(s)

    #stats.uk-card.uk-card-default.uk-card-body
      template(v-for='check in status.checks')
        .header.uk-flex
          .bubble.success.uk-margin-right(v-if='check.ok')
          .bubble.error.uk-margin-right(v-else)

          div
            p.uk-margin-remove.uk-text-bold.uk-text-emphasis {{ check.name }}
            p.uk-margin-remove.uk-text-muted.uk-text-small(v-if='check.down_since')
              span(:uk-tooltip='`title: ${$helpers.datetime(check.down_since)}`') {{ $helpers.ago(check.down_since) }}

        .stats
          Timeline(:outages='check.stats || {}')
  div(v-else-if="!enabled")
    p The status page was not enabled on this Defcon instance.
</template>

<script>
import Timeline from '~/components/status/Timeline.vue';

export default {
  components: {
    Timeline,
  },

  inject: ['store', '$publicHttp', '$helpers'],

  data: () => ({
    enabled: true,
    status: undefined,
  }),

  async mounted() {
    this.refresh();
  },

  methods: {
    refresh() {
      this.$publicHttp().get('/api/status-page')
        .then((response) => {
          this.status = response.data;
        })
        .catch((e) => {
          if (e.response.status === 404) {
            this.enabled = false;
          }
        });
    },
  },
};
</script>

<style lang="scss" scoped>
@import 'uikit/src/scss/variables-theme.scss';

h3 {
  color: white;
}

#stats {
  display: grid;
  grid-template-columns: auto 1fr;
  column-gap: 32px;
  row-gap: 32px;

  .header {
    align-items: center;
  }

  .stats {
    display: flex;
    flex-direction: column;
    justify-content: center;
    flex: 1;
  }
}

@media (max-width: $breakpoint-medium) {
  #stats {
    grid-template-columns: 1fr;
    row-gap: 0;

    .header {
      margin-bottom: 12px;
    }

    .stats {
      margin-bottom: 32px;
    }
  }
}
</style>
