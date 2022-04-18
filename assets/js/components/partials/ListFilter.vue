<template lang="pug">
.filter
  label {{ title }}
  CustomSelect(
    v-on='$listeners',
    v-bind='$attrs',
    :options='options',
    :placeholder='placeholder',
    track-by='slug',
    label='label'
  )
</template>

<script>
import CustomSelect from '@/components/partials/CustomSelect.vue';

export default {
  components: {
    CustomSelect,
  },

  props: {
    title: {
      type: String,
      required: true,
    },
    placeholder: {
      type: String,
      required: true,
    },
    options: {
      type: Array,
      required: true,
    },
  },

  methods: {
    setInput(value) {
      this.$emit('input', value);
    },
  },
};
</script>

<style lang="scss" scoped>
.filter {
  margin-bottom: 32px;

  label {
    font-size: 0.7em;
    color: #525252;
    text-transform: uppercase;
  }
}
</style>
