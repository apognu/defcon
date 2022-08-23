<template lang="pug">
div(v-if='outage')
  #header.uk-flex.uk-margin-bottom
    .uk-flex-1
      p.uk-margin-remove.uk-text-small.uk-text-bolder.uk-text-uppercase Incident
      h2.uk-margin-remove {{ outage.check.name }}

    p#acknowledgement.uk-margin-remove(v-if='outage.acknowledged_by')
      | Acknowledged by
      img.uk-comment-avatar(:src="avatar(outage.acknowledged_by.email)")
      | {{ outage.acknowledged_by.name }}

  .heading.uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin-bottom
    .uk-flex.uk-flex-middle
      div
        .bubble.success.uk-margin-right(v-if='outage.ended_on')
        .bubble.error.uk-margin-right(v-else)

      .uk-flex-1
        p.uk-text-bold.uk-text-emphasis.uk-margin-remove {{ outage.check.name }}
        p.uk-margin-remove.uk-text-muted(class="uk-visible@m") {{ outage.check.uuid }}

      p(v-if="!outage.acknowledged_by"): a(@click="acknowledge()") Acknowledge

      router-link.uk-margin-left(:to='{ name: "checks.view", params: { uuid: outage.check.uuid } }')
        span(uk-icon='icon: search')

  .uk-child-width-expand.uk-grid-match.uk-margin-bottom(uk-grid)
    div
      .uk-card.uk-card-default.uk-card-small.uk-text-center
        .uk-card-header
          h3.uk-h6.uk-text-uppercase.uk-text-muted Started at
        .uk-card-body
          p.uk-text.bold.uk-text-emphasis(:uk-tooltip='`title: ${$helpers.datetime(outage.started_on)}`') {{ $helpers.ago(outage.started_on) }}

    div
      .uk-card.uk-card-default.uk-card-small.uk-text-center
        .uk-card-header
          h3.uk-h6.uk-text-uppercase.uk-text-muted Lasted
        .uk-card-body
          p.uk-text-bold.uk-text-success(v-if='outage.ended_on', :uk-tooltip='`title: ${$helpers.datetime(outage.ended_on)}`') {{ $helpers.humanize(lasted(outage)) }}
          p.uk-text-bold.uk-text-warning(v-else) Ongoing

  .uk-card.uk-card-default.uk-card-small.uk-card-body.uk-margin
    h3.uk-card-title Timeline

    textarea.uk-textarea.uk-margin-small-bottom(
      rows='1',
      placeholder="Write a comment in markdown...",
      v-model='comment',
      @keydown.ctrl.enter='addComment',
      @input="resizeCommentTextArea()"
      ref="comment"
    )

    .uk-text-right
      button.uk-button.uk-button-small.uk-button-primary(@click='comment')
        span(uk-icon='icon: comment')

    Timeline(:updatedAt='timelineUpdatedAt')

  .uk-card.uk-card-default.uk-card-body.uk-margin(v-if='events')
    h3 Latest events

    Events(:events='events')
</template>

<script>
import { MD5 } from 'crypto-js';
import UIkit from 'uikit';

import Spec from '~/components/checks/Spec.vue';
import Events from '~/components/outages/Events.vue';
import Timeline from '~/components/outages/Timeline.vue';

export default {
  components: {
    Spec,
    Events,
    Timeline,
  },

  inject: ['$http', '$helpers'],

  data: () => ({
    outage: undefined,
    events: undefined,
    comment: '',
    timelineUpdatedAt: new Date().toString(),
  }),

  async mounted() {
    this.refresh();
  },

  methods: {
    refresh() {
      this.$http().get(`/api/outages/${this.$route.params.uuid}`).then((response) => {
        this.outage = response.data;
      });

      this.$http().get(`/api/outages/${this.$route.params.uuid}/events?limit=20`).then((response) => {
        this.events = response.data;
      });
    },

    lasted(outage) {
      return this.$moment.duration(this.$moment(outage.ended_on).diff(this.$moment(outage.started_on)));
    },

    resizeCommentTextArea() {
      const element = this.$refs.comment;
      const rows = (this.comment.match(/\n/g) || []).length;

      element.rows = rows + 1;
    },

    acknowledge() {
      this.$http().post(`/api/outages/${this.$route.params.uuid}/acknowledge`)
        .then(() => this.refresh())
        .catch((e) => {
          this.$helpers.error(`${e.message}: ${e.response.data.details}`);
        });
    },

    avatar(email) {
      return `https://www.gravatar.com/avatar/${MD5(email)}`;
    },

    addComment() {
      this.$http()
        .put(`/api/outages/${this.$route.params.uuid}/comment`, {
          comment: this.comment,
        })
        .then(() => {
          UIkit.notification('<span uk-icon="icon: pencil"></span> Your comment on this outage was saved...');

          this.timelineUpdatedAt = new Date().toString();
          this.comment = '';
        })
        .catch((e) => {
          this.$helpers.error(`${e.message}: ${e.response.data.details}`);
        });
    },
  },
};
</script>

<style lang="scss" scoped>
@import 'uikit/src/scss/variables-theme.scss';

@media (max-width: $breakpoint-medium) {
  #header {
    flex-direction: column;
  }
}

textarea {
  font-family: monospace;
  font-size: 0.9rem;
}

#acknowledgement {
  img {
    display: inline-block;
    width: 32px;
    height: 32px;
    min-width: auto;
    margin: 0 8px;
  }
}
</style>
