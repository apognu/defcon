<template lang="pug">
div(uk-grid, class='uk-child-width-1-2@s uk-child-width-1-4@m')
  Attribute(label='URL') {{ spec.url }}
  Attribute(label='Headers') {{ spec.headers }}
  Attribute(label='Timeout') {{ spec.timeout }}
  Attribute(label='Expected status') {{ spec.code }}
  Attribute(label='Expected content') {{ spec.content }}
  Attribute(label='Expected digest') {{ spec.digest }}
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
