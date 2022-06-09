<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Script name
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='spec.script',
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
