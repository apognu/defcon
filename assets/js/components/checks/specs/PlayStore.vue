<template lang="pug">
.uk-child-width-1-4(uk-grid)
  Attribute(label='Application ID') {{ spec.app_id }}
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
