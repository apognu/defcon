<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Host
      .uk-form-controls
        input.uk-input(type='text', v-model='spec.host')
</template>

<script>
export default {
  props: {
    spec: {
      type: Object,
      required: true,
    },
  },

  methods: {
    serialize() {
      return this.spec;
    },
  },
};
</script>
