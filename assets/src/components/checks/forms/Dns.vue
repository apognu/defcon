<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label DNS record type
      .uk-form-controls
        Field(:model='v$.spec.record')
          input.uk-input(
            type='text',
            v-model='v$.spec.record.$model',
            @keyup.enter='$emit("enter")'
          )

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
      label.uk-form-label Value
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='spec.value',
          @keyup.enter='$emit("enter")'
        )
</template>

<script>
import { useVuelidate } from '@vuelidate/core';
import { helpers, required } from '@vuelidate/validators';

import { dnsRecordType } from '~/components/validators';

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
      record: {
        required,
        dns_record_type: helpers.withMessage('Should be one of the supported DNS record types', dnsRecordType),
      },
      domain: { required },
    },
  }),

  methods: {
    serialize() {
      return this.spec;
    },
  },
};
</script>
