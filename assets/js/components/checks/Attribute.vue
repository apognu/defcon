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
      if (this.$slots.default[0].text === '') {
        return '-';
      }

      return this.$slots.default[0].text;
    },
  },
};
</script>

<style lang="scss">
</style>
