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

    .uk-margin
      label.uk-form-label Port
      .uk-form-controls
        Field(:model='v$.spec.port')
          input.uk-input(
            type='text',
            v-model.number='v$.spec.port.$model',
            @keyup.enter='$emit("enter")'
          )

    .uk-margin
      label.uk-form-label Timeout
      .uk-form-controls
        Field(:model='v$.spec.timeout')
          input.uk-input(
            type='text',
            v-model='v$.spec.timeout.$model',
            @keyup.enter='$emit("enter")'
          )
</template>

<script>
import { useVuelidate } from '@vuelidate/core';
import { required, integer } from '@vuelidate/validators';

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
      host: { required },
      port: { required, integer },
      timeout: { duration },
    },
  }),

  methods: {
    serialize() {
      return this.spec;
    },
  },
};
</script>
