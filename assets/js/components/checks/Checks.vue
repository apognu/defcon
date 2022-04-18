<template lang="pug">
div
  h2 Checks

  p.uk-text-right
    router-link.uk-button.uk-button-primary.uk-button-small(
      :to='{ name: "checks.new" }'
    ) New check

  .uk-child-width-1-3.uk-margin-bottom(uk-grid)
    .uk-form-stacked
      label.uk-form-label Search
      .uk-form-control
        .uk-inline.uk-width-1-1
          span.uk-form-icon(uk-icon='icon: search')
          input.uk-input(
            type='text',
            placeholder='Check name',
            v-model='terms',
            @keyup.enter='search()'
          )

    .uk-form-stacked
      label.uk-form-label Group
      .uk-form-controls
        select.uk-select(v-model='filters.group')
          option(:value='undefined') -
          option(v-for='option in groups_options', :value='option.slug') {{ option.label }}

    .uk-form-stacked
      label.uk-form-label Check state
      .uk-form-controls
        select.uk-select(v-model='filters.state')
          option(v-for='option in state_options', :value='option.slug') {{ option.label }}

  .uk-card.uk-card-default.uk-card-body(v-if='filteredChecks.length > 0')
    table.uk-table.uk-table-middle
      tbody
        tr(v-for='check in filteredChecks')
          td.uk-table-shrink
            .bubble.success(v-if='check.status')
            .bubble.error(v-else)

          td
            p.uk-margin-remove
              span.uk-text-bold.uk-text-emphasis(
                :class='{ "uk-text-warning": !check.enabled }'
              ) {{ check.name }}
              span.uk-margin-left.uk-text-muted(v-if='check.group') {{ check.group.name }}
            p.uk-margin-remove.uk-text-muted.uk-text-small {{ check.uuid }}

          td.uk-table-shrink.uk-text-nowrap.uk-text-right
            span.checkkind {{ check.spec.kind | checkkind() }}

          td.actions
            ul.uk-iconnav
              router-link(
                :to='{ name: "checks.edit", params: { uuid: check.uuid } }',
                tag='li'
              )
                a(uk-icon='icon: pencil')

              router-link(
                :to='{ name: "checks.view", params: { uuid: check.uuid } }',
                tag='li'
              )
                a(uk-icon='icon: search')

  .uk-placeholder(v-else) No checks were found for the provided filters.
</template>

<script>
import axios from 'axios';

export default {
  data: () => ({
    checks: [],
    groups: [],
    terms: '',
    filters: {
      search: '',
      group: undefined,
      state: 'enabled',
    },
  }),

  computed: {
    filteredChecks() {
      const regex = new RegExp(`.*${this.filters.search}.*`, 'gi');

      return this.checks.filter((check) => check.name.match(regex));
    },

    groups_options() {
      return this.groups.map((group) => ({
        slug: group.uuid,
        label: group.name,
      }));
    },

    state_options() {
      return [
        { slug: 'enabled', label: 'Enabled' },
        { slug: 'all', label: 'All' },
      ];
    },
  },

  watch: {
    'filters.group': function groupWatcher() {
      this.refresh();
    },

    'filters.state': function stateWatcher() {
      this.refresh();
    },
  },

  async mounted() {
    this.refresh();

    axios.get('/api/groups').then((response) => {
      this.groups = response.data;
    });
  },

  methods: {
    refresh() {
      let all = 'all=false';
      if (this.filters.state === 'all') {
        all = 'all=true';
      }

      if (this.filters.group === undefined) {
        axios.get(`/api/checks?${all}`).then((response) => {
          this.checks = response.data;
        });
      } else {
        axios
          .get(`/api/checks?${all}&group=${this.filters.group}`)
          .then((response) => {
            this.checks = response.data;
          });
      }
    },

    search() {
      this.filters.search = this.terms;
    },
  },
};
</script>
