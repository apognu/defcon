<template lang="pug">
div
  h2 Alerters

  p.uk-text-right
    router-link.uk-button.uk-button-primary.uk-button-small(
      :to='{ name: "alerters.new" }'
    ) New alerter

  .uk-card.uk-card-default.uk-card-body
    table.uk-table.uk-table-middle
      tr(v-for='alerter in alerters')
        td
          p.uk-margin-remove.uk-text-bold.uk-text-emphasis {{ alerter.name }}
          p.uk-margin-remove.uk-text-muted.uk-text-small {{ alerter.uuid }}

        td.uk-table-shrink
          span.checkkind {{ alerter.kind }}

        td.actions
          ul.uk-iconnav
            router-link(
              :to='{ name: "alerters.edit", params: { uuid: alerter.uuid } }',
              tag='li'
            )
              a(uk-icon='icon: pencil')

            li
              a.uk-text-danger(
                @click='deleteAlerter(alerter.uuid)',
                uk-icon='icon: trash'
              )

  #modal-delete-alerter(uk-modal, ref='modal_delete')
    .uk-modal-dialog
      .uk-modal-header
        h3.uk-modal-title Delete this alerter?
      .uk-modal-body
        p This will delete this alerter and disassociate all checks on which it was configured. This action cannot be undone.
      .uk-modal-footer.uk-text-right
        button.uk-button.uk-button-default.uk-modal-close.uk-margin-left Cancel
        button.uk-button.uk-button-danger.uk-modal-close.uk-margin-left Delete
</template>

<script>
import UIkit from 'uikit';
import axios from 'axios';

export default {
  data: () => ({
    alerters: [],
  }),

  async mounted() {
    this.refresh();
  },

  methods: {
    refresh() {
      axios.get('/api/alerters').then((response) => {
        this.alerters = response.data;
      });
    },

    deleteAlerter(uuid) {
      UIkit.modal
        .confirm(
          'Are you certain you want to delete this alerter? This will permanently delete the alerter and disassociate all checks on which it was configured. This action cannot be undone.',
          { labels: { ok: 'Delete', cancel: 'Cancel' } },
        )
        .then(() => {
          axios.delete(`/api/alerters/${uuid}`).then(() => {
            this.refresh();
          });
        });
    },
  },
};
</script>
