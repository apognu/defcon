<template lang="pug">
div
  h2 Users

  p.uk-text-right
    router-link.uk-button.uk-button-primary.uk-button-small(
      :to='{ name: "users.new" }'
    ) New user

  .uk-card.uk-card-default.uk-card-body(v-if='users.length > 0')
    table.uk-table.uk-table-middle
      tr(v-for='user in users')
        td.uk-table-shrink: img.uk-comment-avatar(:src='avatar(user.email)')
        td
          p.uk-margin-remove.uk-text-bold.uk-text-emphasis.uk-margin-right
            | {{ user.name }}
            span.uk-label(v-if='self(user)') You
          p.uk-margin-remove.uk-text-muted.uk-text-small(class='uk-visible@m') {{ user.email }}

        td.actions
          ul.uk-iconnav
            router-link(
              :to='{ name: "users.edit", params: { uuid: user.uuid } }',
              tag='li'
            )
              a(uk-icon='icon: pencil')

            li(v-if='!self(user)')
              a.uk-text-danger(
                uk-icon='icon: trash',
                @click='deleteGroup(user.uuid)'
              )

  .uk-placeholder(v-else) Defcon does not have any users configured.
</template>

<script>
import { MD5 } from 'crypto-js';
import UIkit from 'uikit';

export default {
  inject: ['store', '$http'],

  data: () => ({
    users: [],
  }),

  async mounted() {
    this.refresh();
  },

  methods: {
    self(user) {
      return this.store.identity && user.uuid === this.store.identity.uuid;
    },

    avatar(email) {
      return `https://www.gravatar.com/avatar/${MD5(email)}`;
    },

    refresh() {
      this.$http().get('/api/users').then((response) => {
        this.users = response.data;
      });
    },

    deleteGroup(uuid) {
      UIkit.modal
        .confirm(
          'Are you certain you want to delete this user? This action cannot be undone.',
          { labels: { ok: 'Delete', cancel: 'Cancel' } },
        )
        .then(() => {
          this.$http().delete(`/api/users/${uuid}`).then(() => {
            this.refresh();
          });
        });
    },
  },
};
</script>

<style lang="scss">
.uk-comment-avatar {
  width: 50px;
  min-width: 50px;
  height: 50px;
  border-radius: 25px;
}
</style>
