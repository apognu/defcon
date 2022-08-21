<template lang="pug">
div
  aside#sidebar.uk-card.uk-card-default.uk-card-body.uk-padding-small(class='uk-visible@m')
    header
      h1.uk-h3 Defcon

      ul#menu.uk-nav.uk-nav-default
        Sidenav

  aside#sidebar.uk-card.uk-card-default.uk-card-body.uk-padding-small(class='uk-hidden@m')
    header
      .uk-flex.uk-flex-middle
        a.uk-margin-right(href='#menu', uk-toggle)
          span(uk-icon='icon: menu; ratio: 1.5')

        h1.uk-margin-remove.uk-h3.uk-width-1-1 Defcon

      #menu(uk-offcanvas='mode: reveal; overlay: true', ref='menu')
        .uk-offcanvas-bar
          ul#menu.uk-nav.uk-nav-default
            Sidenav(:mobile='true', @close='closeMenu')

  #main
    router-view.uk-container-large.uk-margin-auto.uk-padding
</template>

<script>
import UIkit from 'uikit';

import Sidenav from '~/components/partials/Sidenav.vue';

export default {
  components: {
    Sidenav,
  },

  inject: ['store', '$http', '$helpers'],

  data: () => ({
    title: '',
    refresher: undefined,
  }),

  async mounted() {
    this.refresh();

    this.$http().get('/api/-/me').then((response) => {
      this.store.setIdentity(response.data);
    });

    this.refresher = setInterval(this.refresh, 5000);
  },

  unmounted() {
    clearInterval(this.refresher);
  },

  methods: {
    refresh() {
      this.$http().get('/api/status').then((response) => {
        this.store.setStatus(response.data.ok);
        this.store.setIncidentCount(response.data.outages.global);
        this.store.setStatusPage(response.data.status_page);
      });
    },

    closeMenu() {
      UIkit.offcanvas(this.$refs.menu).hide();
    },
  },
};
</script>

<style lang="scss">
@import 'uikit/src/scss/variables-theme.scss';
@import '@/../css/colors.scss';

$sidebar-width: 300px;
$sidebar-padding: 16px;

#app {
  min-height: 100vh;
}

#menu {
  .uk-badge {
    background: $error important;
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

@media (min-width: $breakpoint-medium) {
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
        background: $error  !important;
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
