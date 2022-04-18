<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Host
      .uk-form-controls
        input.uk-input(type='text', v-model='spec.host')

    .uk-margin
      label.uk-form-label Port
      .uk-form-controls
        input.uk-input(type='text', v-model.number='spec.port')

    .uk-margin
      label.uk-form-label Message to send
      .uk-form-controls
        input.uk-input(type='text', v-model='spec.message')

    .uk-margin
      label.uk-form-label Expected content
      .uk-form-controls
        input.uk-input(type='text', v-model='spec.content')

    .uk-margin
      label.uk-form-label Timeout
      .uk-form-controls
        input.uk-input(type='text', v-model='spec.timeout')
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
