<template lang="pug">
div(v-if='store.config')
  .uk-alert-warning.uk-alert-callout.uk-margin.uk-padding-small(v-if='!store.config.dms.enable')
    | The Dead Man Switch process is not enabled in your configuration of Defcon, this check will never run.

  div(uk-grid, class='uk-child-width-1-2@s uk-child-width-1-4@m')
    Attribute(label='Stale after') {{ spec.stale_after }}

  .uk-margin-top
    Attribute(label='Check-in URL') {{ url }}
</template>

<script>
import Attribute from '~/components/checks/Attribute.vue';

export default {
  inject: ['store'],

  components: {
    Attribute,
  },

  props: {
    spec: {
      type: Object,
      default: undefined,
    },
  },

  computed: {
    url() {
      return `${this.store.config.domain}${this.spec.checkin_url}`;
    },
  },
};
</script>
