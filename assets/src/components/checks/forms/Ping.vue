<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Host
      .uk-form-controls
        Field(:model='v$.spec.host')
          input.uk-input(
            type='text',
            v-model='v$.spec.host.$model',
            @keyup.enter='$emit("enter")'
          )
</template>

<script>
import { useVuelidate } from '@vuelidate/core';
import { required } from '@vuelidate/validators';

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
      host: { required },
    },
  }),

  methods: {
    serialize() {
      return this.spec;
    },
  },
};
</script>
