<template lang="pug">
div(v-if='store.authenticated !== undefined')
  PublicApp(v-if='$route.meta.public')
  App(v-else-if='store.authenticated')
  Login(v-else)
</template>

<script>
import PublicApp from '~/components/PublicApp.vue';
import App from '~/components/App.vue';
import Login from '~/components/session/Form.vue';

export default {
  components: {
    PublicApp,
    App,
    Login,
  },

  inject: ['store', '$http'],

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

  async mounted() {
    this.$http().get('/api/-/me')
      .then(() => {
        this.store.authenticated = true;
      })
      .catch(() => {
        this.store.revokeToken();
      });
  },
};
</script>
