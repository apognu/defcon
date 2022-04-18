<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Application ID
      .uk-form-controls
        input.uk-input(type='text', v-model='spec.app_id')
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
