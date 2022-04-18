<template lang="pug">
.uk-child-width-1-4(uk-grid)
  Attribute(label='Domain') {{ spec.domain }}
  Attribute(label='Window') {{ spec.window }}
  Attribute(label='Attribute') {{ spec.attribute }}
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
