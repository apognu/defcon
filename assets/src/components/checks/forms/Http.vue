<template lang="pug">
div
  .uk-form-horizontal
    .uk-margin
      label.uk-form-label URL
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='spec.url',
          @keyup.enter='$emit("enter")'
        )

    .uk-margin
      label.uk-form-label Timeout
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='spec.timeout',
          @keyup.enter='$emit("enter")'
        )

    .uk-margin
      label.uk-form-label Headers JSON map
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='headers',
          @keyup.enter='$emit("enter")'
        )

    .uk-margin
      label.uk-form-label Status code
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model.number='spec.code',
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
        input.uk-input(
          type='text',
          v-model='spec.digest',
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
        input.uk-input(
          type='text',
          v-model='spec.duration',
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

  data: () => ({
    headers: {},
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
