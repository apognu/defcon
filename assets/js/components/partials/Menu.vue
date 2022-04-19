<template lang="pug">
div
  template(v-for='item in items')
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
export default {
  props: {
    mobile: {
      type: Boolean,
      default: false,
    },

    outages: {
      type: Number,
      required: true,
    },
  },

  data: () => ({
    items: [
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
    ],
  }),

  computed: {
    incidents() {
      if (this.outages > 0) {
        return this.outages;
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
