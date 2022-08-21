<template lang="pug">
div
  #main-public
    router-view.uk-container-large.uk-margin-auto.uk-padding
</template>

<script>
export default {
  data: () => ({
    title: '',
  }),
};
</script>

<style lang="scss">
@import 'uikit/src/scss/variables-theme.scss';

#app {
  min-height: 100vh;
}
</style>
