<template lang="pug">
div
  div(uk-grid, class='uk-child-width-1-2@s uk-child-width-1-4@m')
    Attribute(label='Stale after') {{ spec.stale_after }}

  .uk-margin-top
    Attribute(label='Check-in URL') {{ spec.checkin_url }}
</template>

<script>
import Attribute from '@/components/checks/Attribute.vue';

export default {
  components: {
    Attribute,
  },

  props: {
    spec: {
      type: Object,
      default: undefined,
    },
  },
};
</script>
