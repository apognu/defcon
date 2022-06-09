<template lang="pug">
App(v-if='store.accessToken')
Login(v-else)
</template>

<script>
import App from '~/components/App.vue';
import Login from '~/components/login/Form.vue';

export default {
  components: {
    App,
    Login,
  },

  inject: ['store'],

  watch: {
    'store.getTitle': function watchTitle(title) {
      document.title = title;
    },

    $route(to) {
      this.store.setTitle(to.meta.title);
    },
  },

  created() {
    this.store.setTitle(this.$route.meta.title);
  },
};
</script>
