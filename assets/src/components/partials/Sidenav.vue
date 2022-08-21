<template lang="pug">
div
  .uk-comment(v-if='store.identity')
    .uk-comment-header
      .uk-grid-medium.uk-flex-middle(uk-grid)
        .uk-width-auto
          img.uk-comment-avatar(:src='avatar')
        #identity-info.uk-width-expand.uk-margin-remove
          h4.uk-comment-title.uk-margin-remove {{ store.identity.name }}
          ul.uk-comment-meta
            li {{ store.identity.email }}

  template(v-for='item in items')
    template(v-if='!item.predicate || item.predicate(store)')
      hr(v-if='item === "divider"')

      router-link(
        v-else,
        :to='{ name: item.route }',
        :exact-active-class='item.exact ? "active" : null',
        :active-class='!item.exact ? "active" : null',
        @click.native='close'
      )
        span.uk-margin-right(:uk-icon='`icon: ${item.icon}`')
        | {{ item.label }}
        span.uk-badge.uk-margin-small-left(
          v-if='item.badge && badge(item.badge)'
        ) {{ badge(item.badge) }}
</template>

<script>
import { MD5 } from 'crypto-js';

export default {
  inject: ['store'],

  props: {
    mobile: {
      type: Boolean,
      default: false,
    },
  },

  data: () => ({
    items: [
      'divider',
      {
        label: 'Dashboard',
        icon: 'home',
        route: 'home',
        exact: true,
      },
      {
        label: 'Incidents',
        icon: 'warning',
        route: 'outages',
        badge: 'incidents',
      },
      { label: 'History', icon: 'history', route: 'outages.history' },
      'divider',
      { label: 'Checks', icon: 'check', route: 'checks' },
      { label: 'Groups', icon: 'folder', route: 'groups' },
      { label: 'Alerters', icon: 'bell', route: 'alerters' },
      'divider',
      {
        label: 'Status page',
        icon: 'world',
        route: 'statuspage',
        predicate: (store) => store.statusPage,
      },
      { label: 'Users', icon: 'users', route: 'users' },
      { label: 'Settings', icon: 'cog', route: 'settings' },
      { label: 'Sign out', icon: 'sign-out', route: 'logout' },
    ],
  }),

  computed: {
    avatar() {
      return `https://www.gravatar.com/avatar/${MD5(this.store.identity.email)}`;
    },

    incidents() {
      if (this.store.incidents > 0) {
        return this.store.incidents;
      }

      return false;
    },
  },

  methods: {
    badge(key) {
      return this[key];
    },

    close() {
      this.$emit('close');
    },
  },
};
</script>

<style lang="scss">
#identity-info {
  padding-left: 16px;
}

.uk-comment-avatar {
  width: 50px;
  height: 50px;
  border-radius: 25px;
}
</style>
