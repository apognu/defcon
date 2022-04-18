<template lang="pug">
div
  h2 Groups

  p.uk-text-right
    router-link.uk-button.uk-button-primary.uk-button-small(
      :to='{ name: "groups.new" }'
    ) New group

  .uk-card.uk-card-default.uk-card-body(v-if='groups.length > 0')
    table.uk-table.uk-table-middle
      tr(v-for='group in groups')
        td
          p.uk-margin-remove.uk-text-bold.uk-text-emphasis {{ group.name }}
          p.uk-margin-remove.uk-text-muted.uk-text-small(class='uk-visible@m') {{ group.uuid }}

        td.actions
          ul.uk-iconnav
            router-link(
              :to='{ name: "groups.edit", params: { uuid: group.uuid } }',
              tag='li'
            )
              a(uk-icon='icon: pencil')

            li
              a.uk-text-danger(
                uk-icon='icon: trash',
                @click='deleteGroup(group.uuid)'
              )

  .uk-placeholder(v-else) Defcon does not have any groups configured.
</template>

<script>
import UIkit from 'uikit';
import axios from 'axios';

export default {
  data: () => ({
    groups: [],
  }),

  async mounted() {
    this.refresh();
  },

  methods: {
    refresh() {
      axios.get('/api/groups').then((response) => {
        this.groups = response.data;
      });
    },

    deleteGroup(uuid) {
      UIkit.modal
        .confirm(
          'Are you certain you want to delete this group? This will permanently delete the group and disassociate all checks on which it was configured. This action cannot be undone.',
          { labels: { ok: 'Delete', cancel: 'Cancel' } },
        )
        .then(() => {
          axios.delete(`/api/groups/${uuid}`).then(() => {
            this.refresh();
          });
        });
    },
  },
};
</script>
