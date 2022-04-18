<template lang="pug">
multiselect(
  v-bind='$attrs',
  v-on='$listeners',
  :multiple='multiple',
  :close-on-select='isCloseOnSelect'
)
  span(slot='noResult') No result found
</template>

<script>
import Multiselect from 'vue-multiselect';

export default {
  components: {
    Multiselect,
  },

  inheritAttrs: false,

  props: {
    multiple: Boolean,
  },

  computed: {
    isCloseOnSelect() {
      return !this.multiple;
    },
  },
};
</script>
