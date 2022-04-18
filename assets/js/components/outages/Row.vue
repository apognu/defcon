<template lang="pug">
tr
  td
    .bubble.success(v-if='outage.ended_on')
    .bubble.error(v-else)

  td
    p.uk-margin-remove.uk-text-emphasis.uk-text-bold {{ outage.check.name }}
    p.uk-margin-remove.uk-text-muted.uk-text-small {{ outage.uuid }}
    p.uk-margin-small-top.message(v-if='outage.event.message') {{ outage.event.message }}

  td.uk-table-shrink
    span.checkkind {{ outage.check.spec.kind }}

  td.uk-table-shrink.uk-text-nowrap
    p {{ outage.started_on | moment("from") }}

  td.uk-table-shrink.uk-text-nowrap
    p(v-if='outage.ended_on') {{ lasted(outage) | duration("humanize") }}
    p.uk-text-bold.uk-text-warning(v-else) Ongoing

  td.actions
    ul.uk-iconnav
      router-link(
        :to='{ name: "outages.view", params: { uuid: outage.uuid } }',
        tag='li'
      )
        a(uk-icon='icon: search')
</template>

<script>
export default {
  props: {
    outage: {
      type: Object,
      required: true,
    },
  },

  methods: {
    lasted(outage) {
      return this.$moment(outage.ended_on).diff(
        this.$moment(outage.started_on),
      );
    },
  },
};
</script>
