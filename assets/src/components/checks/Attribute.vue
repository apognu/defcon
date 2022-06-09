<template lang="pug">
div
  dl.uk-description-list
    dt {{ label }}
    dd {{ value }}
</template>

<script>
export default {
  props: {
    label: {
      type: String,
      required: true,
    },
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
</style>
