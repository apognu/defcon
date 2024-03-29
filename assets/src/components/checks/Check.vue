<template lang="pug">
.check(v-if='check')
  p.uk-margin-remove.uk-text-small.uk-text-bolder.uk-text-uppercase Check
  h2.uk-margin-remove-top {{ check.name }}

  .uk-alert.uk-alert-primary(v-if='check && check.enabled && check.on_status_page')
    span.uk-margin-right(uk-icon='icon: world')
    | This check appears on your public status page.

  .uk-alert.uk-alert-warning(v-if='check && !check.enabled')
    span.uk-margin-right(uk-icon='icon: ban')
    | This check is disabled.

  .uk-alert(v-if='check && check.enabled && check.silent')
    span.uk-margin-right(uk-icon='icon: bell')
    | This check is silenced.

  .heading.uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin-bottom
    .uk-flex.uk-flex-middle
      .uk-flex-1
        p.uk-text-bold.uk-text-emphasis.uk-margin-remove {{ check.name }}

      a.uk-margin-left.uk-text-success(v-if='check.silent', @click='silence(false)', uk-tooltip='Unsilence')
        span(uk-icon='icon: bell')
      a.uk-margin-left.uk-text-danger(v-else, @click='silence(true)', uk-tooltip='Silence')
        span(uk-icon='icon: bell')

      a.uk-margin-left.uk-text-danger(v-if='check.enabled', @click='enable(false)', uk-tooltip='Disable')
        span(uk-icon='icon: ban')
      a.uk-margin-left.uk-text-success(v-else, @click='enable(true)', uk-tooltip='Enable')
        span(uk-icon='icon: play')

      router-link.uk-margin-left(:to='{ name: "checks.edit", uuid: check.uuid }')
        span(uk-icon='icon: pencil')

      a.uk-margin-left.uk-text-danger(@click='deleteCheck()')
        span(uk-icon='icon: trash')

  Spec.uk-margin-bottom(:check='check')

  Timeline(:check='check.uuid', :showUptime='true')

  div(uk-grid)
    .uk-width-1-1(class='uk-width-1-2@m')
      .uk-card.uk-card-default(v-if='outages.length > 0')
        .uk-card-header
          h3.uk-card-title Past incidents
        .uk-card-body
          table.uk-table.uk-table-middle
            tbody
              tr(v-for='outage in outages', is='vue:OutageRow', :outage='outage', :key='outage.uuid')

    .uk-width-1-1(class='uk-width-1-2@m')
      .uk-card.uk-card-default(v-if='events.length > 0')
        .uk-card-header
          h3.uk-card-title Latest events
        .uk-card-body
          Events(:events='events')
</template>

<script>
import UIkit from 'uikit';

import Spec from '~/components/checks/Spec.vue';
import Timeline from '~/components/dashboard/Timeline.vue';
import OutageRow from '~/components/outages/Row.vue';
import Events from '~/components/outages/Events.vue';

export default {

  components: {
    Spec,
    Timeline,
    OutageRow,
    Events,
  },
  inject: ['$http'],

  data: () => ({
    refresher: undefined,
    check: undefined,
    outages: [],
    events: [],
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
      this.$http().get(`/api/checks/${this.$route.params.uuid}`).then((response) => {
        this.check = response.data;
      });

      this.$http().get(`/api/checks/${this.$route.params.uuid}/outages?limit=10`).then((response) => {
        this.outages = response.data;
      });

      this.$http().get(`/api/checks/${this.$route.params.uuid}/events?limit=10`).then((response) => {
        this.events = response.data;
      });
    },

    lasted(outage) {
      return this.$moment(outage.ended_on).diff(this.$moment(outage.started_on));
    },

    silence(state) {
      this.$http().patch(`/api/checks/${this.$route.params.uuid}`, { silent: state }).then(() => {
        this.refresh();

        if (state) {
          UIkit.notification('<span class="uk-margin-small-right" uk-icon="icon: bell"></span> The check was silenced.');
        } else {
          UIkit.notification('<span class="uk-margin-small-right" uk-icon="icon: bell"></span> The check was unsilenced.');
        }
      });
    },

    enable(state) {
      this.$http().patch(`/api/checks/${this.$route.params.uuid}`, { enabled: state }).then(() => {
        this.refresh();

        if (state) {
          UIkit.notification('<span class="uk-margin-small-right" uk-icon="icon: play"></span> The check was enabled.');
        } else {
          UIkit.notification('<span class="uk-margin-small-right" uk-icon="icon: ban"></span> The check was disabled.');
        }
      });
    },

    deleteCheck() {
      UIkit.modal
        .confirm('Are you certain you want to delete this check? This will permanently delete the check. This action cannot be undone.', { labels: { ok: 'Delete', cancel: 'Cancel' } })
        .then(() => {
          this.$http().delete(`/api/checks/${this.$route.params.uuid}?delete=true`).then(() => {
            this.$router.push({ name: 'checks' });
          });
        });
    },
  },
};
</script>
