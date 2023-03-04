<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Stale after
      .uk-form-controls
        Field(:model='v$.spec.stale_after')
          input.uk-input(
            type='text',
            v-model='v$.spec.stale_after.$model',
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
      stale_after: { required, duration },
    },
  }),

  methods: {
    serialize() {
      return this.spec;
    },
  },
};
</script>
