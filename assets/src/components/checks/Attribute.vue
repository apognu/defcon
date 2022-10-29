<template lang="pug">
div
  dl.uk-description-list
    dt {{ label }}
    dd(:class="{ code: code }") {{ value }}
</template>

<script>
export default {
  props: {
    label: {
      type: String,
      required: true,
    },

    code: {
      type: Boolean,
      required: false,
      default: false
    }
  },

  computed: {
    value() {
      const text = this.$slots.default()[0];

      if (text.children === '') {
        return '-';
      }

      return text.children;
    },
  },
};
</script>

<style lang="scss">
.code {
  padding: 6px 8px;
  background: var(--gray-dark);
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.9em;
  color: white;
}
</style>
