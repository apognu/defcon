<template lang="pug">
.uk-child-width-1-4(uk-grid)
  Attribute(label='Host') {{ spec.host }}
  Attribute(label='Port') {{ spec.port }}
  Attribute(label='Message') {{ spec.message }}
  Attribute(label='Content') {{ spec.content }}
  Attribute(label='Timeout') {{ spec.timeout }}
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
