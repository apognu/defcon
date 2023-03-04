<template lang="pug">
.form-field-wrapper(:class='{ "error": model.$error }')
  slot
  .uk-text-meta.uk-form-danger(v-if='model.$error')
    span.uk-display-block(v-for='error of model.$errors', :key='error.key') {{ error.$message }}
</template>

<script>
export default {
  props: {
    model: {
      type: Object,
      required: true,
    },
  },
};
</script>
