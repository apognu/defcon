<template lang="pug">
div
  h2 Checks

  p.uk-text-right
    router-link.uk-button.uk-button-primary.uk-button-small(:to='{ name: "checks.new" }') New check

  .uk-margin-bottom(uk-grid, class='uk-child-width-1-3@m uk-child-width-1-1@s')
    .uk-form-stacked
      label.uk-form-label Search
      .uk-form-control
        .uk-inline.uk-width-1-1
          span.uk-form-icon(uk-icon='icon: search')
          input.uk-input(type='text', placeholder='Check name', v-model='terms', @keyup.enter='search()')

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
              span.uk-text-bold.uk-text-emphasis
                | {{ check.name }}
                span.uk-margin-small-left(v-if='check.on_status_page', uk-icon='icon: world; ratio: 0.8', uk-tooltip='On status page')
                span.uk-margin-small-left(v-if='check.enabled && check.silent', uk-icon='icon: bell; ratio: 0.8', uk-tooltip='Silenced')
                span.uk-margin-small-left.uk-text-danger(v-if='!check.enabled', uk-icon='icon: ban; ratio: 0.8', uk-tooltip='Disabled')
              span.uk-margin-left.uk-text-muted(v-if='check.group', class='uk-visible@m') {{ check.group.name }}
            p.uk-margin-remove.uk-text-muted.uk-text-small(class='uk-visible@m') {{ check.uuid }}

          td.uk-table-shrink.uk-text-nowrap.uk-text-right(class='uk-visible@m')
            span.checkkind {{ $filters.checkkind(check.spec.kind) }}

          td.actions
            ul.uk-iconnav
              router-link(:to='{ name: "checks.edit", params: { uuid: check.uuid } }', tag='li')
                a(uk-icon='icon: pencil')

              router-link(:to='{ name: "checks.view", params: { uuid: check.uuid } }', tag='li')
                a(uk-icon='icon: search')

  .uk-placeholder(v-else) No checks were found for the provided filters.
</template>

<script>
export default {
  inject: ['$http', '$filters'],

  data: () => ({
    refresher: undefined,
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
      this.$router.push({ query: { ...this.$route.query, group: this.filters.group } });
      this.refresh();
    },

    'filters.state': function stateWatcher() {
      this.$router.push({ query: { ...this.$route.query, state: this.filters.state } });
      this.refresh();
    },
  },

  async mounted() {
    if (this.$route.query.group) {
      this.filters.group = this.$route.query.group;
    }
    if (this.$route.query.state) {
      this.filters.state = this.$route.query.state;
    }
    if (this.$route.query.search) {
      this.terms = this.$route.query.search;
      this.filters.search = this.$route.query.search;
    }

    this.refresh();
    this.refresher = setInterval(this.refresh, 5000);

    this.$http().get('/api/groups').then((response) => {
      this.groups = response.data;
    });
  },

  unmounted() {
    clearInterval(this.interval);
  },

  methods: {
    refresh() {
      let all = 'all=false';
      if (this.filters.state === 'all') {
        all = 'all=true';
      }

      if (this.filters.group === undefined) {
        this.$http().get(`/api/checks?${all}`).then((response) => {
          this.checks = response.data;
        });
      } else {
        this.$http().get(`/api/checks?${all}&group=${this.filters.group}`).then((response) => {
          this.checks = response.data;
        });
      }
    },

    search() {
      this.filters.search = this.terms;
      this.$router.push({ query: { ...this.$route.query, search: this.filters.search } });
    },
  },
};
</script>
