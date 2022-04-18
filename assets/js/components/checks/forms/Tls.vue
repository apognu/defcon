<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Domain
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='spec.domain',
          @keyup.enter='$emit("enter")'
        )

    .uk-margin
      label.uk-form-label Expiration window
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='spec.window',
          @keyup.enter='$emit("enter")'
        )
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
