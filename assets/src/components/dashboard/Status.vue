<template lang="pug">
div(v-if='store.status !== undefined')
  .uk-alert-success.uk-alert-callout.uk-margin.uk-padding-small.uk-border-rounded(v-if='store.status')
    h4.uk-margin-remove.uk-text-center
      span.uk-margin-right(uk-icon='icon: check; ratio: 1.2')
      | Everything is fine

  .uk-alert-danger.uk-alert-callout.uk-margin.uk-padding-small.uk-border-rounded(v-else)
    h4.uk-margin-remove.uk-text-center
      span.uk-margin-right(uk-icon='icon: warning; ratio: 1.2')
      | {{ store.incidents }} active incident(s)
</template>

<script>
export default {
  inject: ['store'],
};
</script>

<style lang="scss" scoped>
h3 {
  color: white;
}
</style>
