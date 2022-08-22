<template lang="pug">
#timeline.uk-margin
  .item.uk-margin-bottom(v-for="item in timeline")
    .dot.avatar.uk-border-circle(v-if='item.author', :style="{ backgroundImage: avatar(item.author.email) }")
    .dot.uk-border-circle(v-else, :class="$filters.timeline(item.kind).class")

    .info.uk-flex.uk-margin-small-bottom
      span.left.author.uk-flex-1.uk-text-bold(v-if="item.author") {{ item.author.name }}
      span.left.uk-flex-1(v-else) {{ $filters.timeline(item.kind).message }}
      span.right.uk-text-small.uk-text-muted(:uk-tooltip='`title: ${$helpers.datetime(item.published_on)}`') {{ $helpers.ago(item.published_on) }}

    .body.uk-margin-bottom.uk-border-rounded.uk-padding-small(v-if="item.content", v-html="item.content")
</template>

<script>
import { MD5 } from 'crypto-js';

export default {
  inject: ['$http', '$helpers', '$filters'],

  props: {
    updatedAt: {
      type: String,
      default: undefined,
    },
  },

  data: () => ({
    timeline: undefined,
  }),

  async mounted() {
    this.refresh();

    this.refresher = setInterval(this.refresh, 5000);
  },

  unmounted() {
    clearInterval(this.refresher);
  },

  watch: {
    updatedAt() {
      this.refresh();
    },
  },

  methods: {
    refresh() {
      this.$http().get(`/api/outages/${this.$route.params.uuid}/timeline`).then((response) => {
        this.timeline = response.data;
      });
    },

    avatar(email) {
      return `url(https://www.gravatar.com/avatar/${MD5(email)})`;
    },
  },
};
</script>

<style lang="scss">
@import '@/../css/colors.scss';

$dot-size: 10px;
$avatar-size: 32px;

#timeline {
  margin-top: 16px;
  margin-left: 32px;
  padding-left: 32px;
  border-left: 4px solid #eaeaea;

  .item {
    position: relative;

    .info {
      align-items: end;

      .left.author {
        margin-top: calc($avatar-size / 4);
      }
    }

    .body {
      background: $background;
    }
  }

  .dot {
    position: absolute;
    width: $dot-size;
    height: $dot-size;
    margin-top: 0.3em;
    background-color: #989898;
    left: calc(($dot-size / -2) - 32px - 2px);

    &.success {
      background-color: $ok;
    }

    &.error {
      background-color: $error;
    }

    &.info {
      background-color: $info;
    }

    &.avatar {
      width: $avatar-size;
      height: $avatar-size;
      margin-top: 0.3em;
      left: calc(($avatar-size / -2) - 32px - 2px);
      background-repeat: no-repeat;
      background-size: cover;
    }
  }
}
</style>
