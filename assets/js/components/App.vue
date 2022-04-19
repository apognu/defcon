<template lang="pug">
#app
  #app-desktop(class='uk-visible@m')
    aside#sidebar.uk-card.uk-card-default.uk-card-body.uk-padding-small
      header
        h1.uk-h3 Defcon

        ul#menu.uk-nav.uk-nav-default
          Menu(:outages='outages')

    #main
      router-view.uk-container-large.uk-margin-auto.uk-padding

  #app-mobile(class='uk-hidden@m')
    aside#sidebar.uk-card.uk-card-default.uk-card-body.uk-padding-small
      header
        .uk-flex.uk-flex-middle
          a.uk-margin-right(href='#menu', uk-toggle)
            span(uk-icon='icon: menu; ratio: 1.5')

          h1.uk-margin-remove.uk-h3.uk-width-1-1 Defcon

        #menu(uk-offcanvas='mode: reveal; overlay: true', ref='menu')
          .uk-offcanvas-bar
            ul#menu.uk-nav.uk-nav-default
              Menu(:mobile='true', :outages='outages', @close='closeMenu')

    #main
      router-view.uk-container-large.uk-margin-auto.uk-padding
</template>

<script>
import UIkit from 'uikit';
import axios from 'axios';

import Menu from '@/components/partials/Menu.vue';

export default {
  components: {
    Menu,
  },

  data: () => ({
    refresher: undefined,
    title: '',
    outages: 0,
  }),

  watch: {
    $route(to) {
      this.setTitle(to.meta.title);
    },
  },

  async mounted() {
    this.refresh();

    setInterval(this.refresh, 5000);
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

    refresh() {
      axios.get('/api/status').then((response) => {
        this.outages = response.data.outages.global;
      });
    },

    closeMenu() {
      UIkit.offcanvas(this.$refs.menu).hide();
    },
  },
};
</script>

<style lang="scss">
$sidebar-width: 300px;
$sidebar-padding: 16px;

#app {
  min-height: 100vh;
}

#menu {
  .uk-badge {
    background: #e55039 !important;
  }

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

#app-desktop {
  aside#sidebar {
    display: block;
    position: fixed;
    top: 0;
    bottom: 0;
    float: left;
    width: $sidebar-width;
    height: 100vh;
    padding: $sidebar-padding;
    background: white;

    #menu {
      .uk-badge {
        background: #e55039 !important;
      }

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
}
</style>
