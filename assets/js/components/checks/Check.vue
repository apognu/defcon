<template lang="pug">
.check
  h2 Check

  .uk-alert.uk-alert-warning(v-if='check && !check.enabled')
    span.uk-margin-right(uk-icon='icon: ban')
    | This check is disabled.

  .uk-alert(v-if='check && check.enabled && check.silent')
    span.uk-margin-right(uk-icon='icon: bell')
    | This check is silenced.

  div(v-if='check')
    .heading.uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin-bottom
      .uk-flex.uk-flex-middle
        .uk-flex-1
          p.uk-text-bold.uk-text-emphasis.uk-margin-remove {{ check.name }}
          p.uk-margin-remove.uk-text-muted {{ check.uuid }}

        a.uk-margin-left.uk-text-success(
          v-if='check.silent',
          @click='silence(false)'
        )
          span(uk-icon='icon: bell')
        a.uk-margin-left.uk-text-danger(v-else, @click='silence(true)')
          span(uk-icon='icon: bell')

        a.uk-margin-left.uk-text-danger(
          v-if='check.enabled',
          @click='enable(false)'
        )
          span(uk-icon='icon: play')
        a.uk-margin-left.uk-text-success(v-else, @click='enable(true)')
          span(uk-icon='icon: play')

        router-link.uk-margin-left(
          :to='{ name: "checks.edit", uuid: check.uuid }'
        )
          span(uk-icon='icon: pencil')

        a.uk-margin-left.uk-text-danger(@click='deleteCheck()')
          span(uk-icon='icon: trash')

    Spec.uk-margin-bottom(:check='check')

    .uk-card.uk-card-default(v-if='outages.length > 0')
      .uk-card-header
        h3.uk-card-title Past outages
      .uk-card-body
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
import UIkit from 'uikit';
import axios from 'axios';

import Spec from '@/components/checks/Spec.vue';
import OutageRow from '@/components/outages/Row.vue';

export default {
  components: {
    Spec,
    OutageRow,
  },

  data: () => ({
    check: undefined,
    outages: [],
  }),

  async mounted() {
    axios.get(`/api/checks/${this.$route.params.uuid}`).then((response) => {
      this.check = response.data;
    });

    axios
      .get(`/api/checks/${this.$route.params.uuid}/outages`)
      .then((response) => {
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

  methods: {
    refresh() {
      axios.get(`/api/checks/${this.$route.params.uuid}`).then((response) => {
        this.check = response.data;
      });
    },

    lasted(outage) {
      return this.$moment(outage.ended_on).diff(
        this.$moment(outage.started_on),
      );
    },

    silence(state) {
      axios
        .patch(`/api/checks/${this.$route.params.uuid}`, { silent: state })
        .then(() => {
          this.refresh();

          if (state) {
            UIkit.notification(
              '<span uk-icon="icon: play"></span> The check was silenced.',
            );
          } else {
            UIkit.notification(
              '<span uk-icon="icon: play"></span> The check was unsilenced.',
            );
          }
        });
    },

    enable(state) {
      axios
        .patch(`/api/checks/${this.$route.params.uuid}`, { enabled: state })
        .then(() => {
          this.refresh();

          if (state) {
            UIkit.notification(
              '<span uk-icon="icon: bell"></span> The check was enabled.',
            );
          } else {
            UIkit.notification(
              '<span uk-icon="icon: bell"></span> The check was disabled.',
            );
          }
        });
    },

    deleteCheck() {
      UIkit.modal
        .confirm(
          'Are you certain you want to delete this check? This will permanently delete the check. This action cannot be undone.',
          { labels: { ok: 'Delete', cancel: 'Cancel' } },
        )
        .then(() => {
          axios
            .delete(`/api/checks/${this.$route.params.uuid}?delete=true`)
            .then(() => {
              this.$router.push({ name: 'checks' });
            });
        });
    },
  },
};
</script>
