<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label URL
      .uk-form-controls
        Field(:model='v$.spec.url')
          input.uk-input(
            type='text',
            v-model='v$.spec.url.$model',
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

    .uk-margin
      label.uk-form-label Headers JSON map
      .uk-form-controls
        Field(:model='v$.headers')
          input.uk-input(
            type='text',
            v-model='v$.headers.$model',
            @keyup.enter='$emit("enter")'
          )

    .uk-margin
      label.uk-form-label Status code
      .uk-form-controls
        Field(:model='v$.spec.code')
          input.uk-input(
            type='text',
            v-model.number='v$.spec.code.$model',
            @keyup.enter='$emit("enter")'
          )

    .uk-margin
      label.uk-form-label Expected content
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='spec.content',
          @keyup.enter='$emit("enter")'
        )

    .uk-margin
      label.uk-form-label Expected SHA256 digest
      .uk-form-controls
        Field(:model='v$.spec.digest')
          input.uk-input(
            type='text',
            v-model='v$.spec.digest.$model',
            @keyup.enter='$emit("enter")'
          )

    .uk-margin
      label.uk-form-label JSON query
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='spec.json_query',
          @keyup.enter='$emit("enter")'
        )

    .uk-margin
      label.uk-form-label Duration
      .uk-form-controls
        Field(:model='v$.spec.duration')
          input.uk-input(
            type='text',
            v-model='v$.spec.duration.$model',
            @keyup.enter='$emit("enter")'
          )
</template>

<script>
import { useVuelidate } from '@vuelidate/core';
import {
  helpers,
  and,
  required,
  integer,
  minLength,
  maxLength,
  url,
} from '@vuelidate/validators';

import { json, duration } from '~/components/validators';

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

  data: () => ({
    headers: {},
  }),

  validations: () => ({
    spec: {
      url: { required, url },
      timeout: { duration },
      code: { integer },
      digest: {
        minLength: helpers.withMessage('Must be a 64-character SHA-256 digest', and(minLength(64), maxLength(64))),
      },
      duration: { duration },
    },
    headers: { json },
  }),

  async mounted() {
    this.headers = JSON.stringify(this.spec.headers);
  },

  methods: {
    serialize() {
      const body = this.spec;

      try {
        body.headers = JSON.parse(this.headers);
      } catch (_) {
        delete body.headers;
      }

      return body;
    },
  },
};
</script>
