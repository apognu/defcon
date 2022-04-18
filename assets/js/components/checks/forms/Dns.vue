<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label DNS record type
      .uk-form-controls
        input.uk-input(type='text', v-model='spec.record')

    .uk-margin
      label.uk-form-label Domain
      .uk-form-controls
        input.uk-input(type='text', v-model='spec.domain')

    .uk-margin
      label.uk-form-label Value
      .uk-form-controls
        input.uk-input(type='text', v-model='spec.value')
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
