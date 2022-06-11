<template lang="pug">
tr
  td.uk-table-shrink
    .bubble.success(v-if='outage.ended_on')
    .bubble.error(v-else)

  td(class='uk-visible@m')
    p.uk-margin-remove.uk-text-emphasis.uk-text-bold {{ outage.check.name }}
    p.uk-margin-remove.uk-text-muted.uk-text-small {{ outage.uuid }}
    p.uk-margin-small-top.message(v-if='outage.event.message') {{ outage.event.message }}

  td(class='uk-hidden@m')
    p.uk-margin-remove.uk-text-emphasis.uk-text-bold {{ outage.check.name }}
    p.uk-margin-remove
      span {{ $helpers.ago(outage.started_on) }}
      span {{ " → " }}
      span(v-if='outage.ended_on') {{ $helpers.humanize(lasted(outage)) }}
      span.uk-text-bold.uk-text-warning(v-else) Ongoing

  td.uk-table-shrink.uk-text-nowrap.uk-text-right
    span.checkkind {{ $filters.checkkind(outage.check.spec.kind) }}

  td.uk-table-shrink.uk-text-nowrap(class='uk-visible@m')
    p(:uk-tooltip='`title: ${$helpers.datetime(outage.started_on)}`') {{ $helpers.ago(outage.started_on) }}

  td.uk-table-shrink.uk-text-nowrap(class='uk-visible@m')
    p(v-if='outage.ended_on', :uk-tooltip='`title: ${this.$helpers.datetime(outage.ended_on)}`') {{ this.$helpers.humanize(lasted(outage)) }}
    p.uk-text-bold.uk-text-warning(v-else) Ongoing

  td.actions
    ul.uk-iconnav
      router-link(:to='{ name: "outages.view", params: { uuid: outage.uuid } }', tag='li')
        a(uk-icon='icon: search')
</template>

<script>
export default {
  inject: ['$filters', '$helpers'],

  props: {
    outage: {
      type: Object,
      required: true,
    },
  },

  methods: {
    lasted(outage) {
      return this.$moment.duration(this.$moment(outage.ended_on).diff(this.$moment(outage.started_on)));
    },
  },
};
</script>
