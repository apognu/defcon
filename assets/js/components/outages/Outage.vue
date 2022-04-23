<template lang="pug">
div(v-if='outage')
  p.uk-margin-remove.uk-text-small.uk-text-bolder.uk-text-uppercase Incident
  h2.uk-margin-remove-top {{ outage.check.name }} - {{ outage.started_on | moment("MMMM Do YYYY, h:mm:ss a") }}

  .heading.uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin-bottom
    .uk-flex.uk-flex-middle
      div
        .bubble.success.uk-margin-right(v-if='outage.ended_on')
        .bubble.error.uk-margin-right(v-else)

      .uk-flex-1
        p.uk-text-bold.uk-text-emphasis.uk-margin-remove {{ outage.check.name }}
        p.uk-margin-remove.uk-text-muted {{ outage.check.uuid }}

      router-link.uk-margin-left(:to='{ name: "checks.view", params: { uuid: outage.check.uuid } }')
        span(uk-icon='icon: search')

  .uk-child-width-expand.uk-grid-match.uk-margin-bottom(uk-grid)
    div
      .uk-card.uk-card-default.uk-card-small.uk-text-center
        .uk-card-header
          h3.uk-h6.uk-text-uppercase.uk-text-muted Started at
        .uk-card-body
          p.uk-text.bold.uk-text-emphasis {{ outage.started_on | moment("from") }}

    div
      .uk-card.uk-card-default.uk-card-small.uk-text-center
        .uk-card-header
          h3.uk-h6.uk-text-uppercase.uk-text-muted Lasted
        .uk-card-body
          p.uk-text-bold.uk-text-success(v-if='outage.ended_on') {{ lasted(outage) | duration("humanize") }}
          p.uk-text-bold.uk-text-warning(v-else) Ongoing

  .uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin
    h3.uk-card-title Comment

    textarea.uk-textarea.uk-form-blank.uk-margin-small-bottom(v-model='outage.comment')
    .uk-text-right
      button.uk-button.uk-button-small(@click='comment') Save comment

  Spec(:check='outage.check')

  .uk-card.uk-card-default.uk-card-body.uk-margin(v-if='events')
    h3 Latest events

    Events(:events='events')
</template>

<script>
import UIkit from 'uikit';
import axios from 'axios';

import Spec from '@/components/checks/Spec.vue';
import Events from '@/components/outages/Events.vue';

export default {
  components: {
    Spec,
    Events,
  },

  data: () => ({
    outage: undefined,
    events: undefined,
  }),

  async mounted() {
    this.refresh();
  },

  methods: {
    refresh() {
      axios.get(`/api/outages/${this.$route.params.uuid}`).then((response) => {
        this.outage = response.data;
      });

      axios.get(`/api/outages/${this.$route.params.uuid}/events?limit=20`).then((response) => {
        this.events = response.data;
      });
    },

    lasted(outage) {
      return this.$moment(outage.ended_on).diff(this.$moment(outage.started_on));
    },

    comment() {
      axios
        .put(`/api/outages/${this.$route.params.uuid}/comment`, {
          comment: this.outage.comment,
        })
        .then(() => {
          UIkit.notification('<span uk-icon="icon: pencil"></span> The comment for this outage was updated..');

          this.refresh();
        })
        .catch((e) => {
          this.$helpers.error(`${e.message}: ${e.response.data.details}`);
        });
    },
  },
};
</script>
