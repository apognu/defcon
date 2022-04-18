<template lang="pug">
.uk-child-width-1-4(uk-grid)
  Attribute(label='Record') {{ spec.record }}
  Attribute(label='Domain') {{ spec.domain }}
  Attribute(label='Value') {{ spec.value }}
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
