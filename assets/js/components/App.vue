<template lang="pug">
div
  aside.uk-card.uk-card-default.uk-card-body.uk-padding-small
    header
      h1.uk-h3 Defcon

      ul#menu.uk-nav.uk-nav-default
        router-link(:to='{ name: "home" }', exact-active-class='active')
          span.uk-margin-right(uk-icon='icon: home')
          | Dashboard
        router-link(:to='{ name: "outages" }', active-class='active')
          span.uk-margin-right(uk-icon='icon: warning')
          | Incidents
        router-link(:to='{ name: "checks" }', active-class='active')
          span.uk-margin-right(uk-icon='icon: check')
          | Checks
        router-link(:to='{ name: "groups" }', active-class='active')
          span.uk-margin-right(uk-icon='icon: folder')
          | Groups
        router-link(:to='{ name: "alerters" }', active-class='active')
          span.uk-margin-right(uk-icon='icon: bell')
          | Alerters

  #main
    router-view.uk-container-large.uk-margin-auto.uk-padding
</template>

<script>
export default {
  data: () => ({
    title: '',
  }),

  watch: {
    $route(to) {
      this.setTitle(to.meta.title);
    },
  },

  created() {
    this.setTitle(this.$route.meta.title);
  },

  methods: {
    setTitle(title) {
      if (title !== undefined) {
        this.title = title;
        /* eslint no-irregular-whitespace: "off" */
        document.title = `Defcon • ${title}`;
      } else {
        document.title = 'Defcon';
      }
    },
  },
};
</script>

<style lang="scss">
$sidebar-width: 300px;
$sidebar-padding: 16px;

aside {
  display: block;
  float: left;
  width: $sidebar-width;
  height: 100vh;
  padding: $sidebar-padding;
  background: white;

  #menu {
    a {
      display: flex;
      align-items: center;
      margin-bottom: 4px;
      padding: 8px 12px;
      border-radius: 4px;
      color: inherit;
      text-decoration: none;

      i.mdi {
        padding-right: 16px;
        font-size: 1.2em;
      }

      &:hover,
      &.active {
        background: #f4f5f8;
      }

      &.active {
        font-weight: bold;
      }
    }
  }
}

#main {
  margin-left: $sidebar-width + (2 * $sidebar-padding);
}
</style>
