<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Domain
      .uk-form-controls
        Field(:model='v$.spec.domain')
          input.uk-input(
            type='text',
            v-model='v$.spec.domain.$model',
            @keyup.enter='$emit("enter")'
          )

    .uk-margin
      label.uk-form-label Expiration window
      .uk-form-controls
        Field(:model='v$.spec.window')
          input.uk-input(
            type='text',
            v-model='v$.spec.window.$model',
            @keyup.enter='$emit("enter")'
          )
</template>

<script>
import { useVuelidate } from '@vuelidate/core';
import { required } from '@vuelidate/validators';

import { duration } from '~/components/validators';
import Field from '~/components/partials/Field.vue';

export default {
  setup: () => ({
    v$: useVuelidate(),
  }),

  components: {
    Field,
  },

  props: {
    spec: {
      type: Object,
      required: true,
    },
  },

  validations: () => ({
    spec: {
      domain: { required },
      window: { duration },
    },
  }),

  methods: {
    serialize() {
      return this.spec;
    },
  },
};
</script>
