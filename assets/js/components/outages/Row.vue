<template lang="pug">
tr
  td.uk-table-shrink(class='uk-visible@m')
    .bubble.success(v-if='outage.ended_on')
    .bubble.error(v-else)

  td(class='uk-visible@m')
    p.uk-margin-remove.uk-text-emphasis.uk-text-bold {{ outage.check.name }}
    p.uk-margin-remove.uk-text-muted.uk-text-small {{ outage.uuid }}
    p.uk-margin-small-top.message(v-if='outage.event.message') {{ outage.event.message }}

  td(class='uk-hidden@m')
    span.bubble.success(v-if='outage.ended_on')
    span.bubble.error(v-else)
    span.uk-margin-left.uk-text-emphasis.uk-text-bold {{ outage.check.name }}

  td.uk-table-shrink.uk-text-nowrap.uk-text-right(class='uk-visible@m')
    span.checkkind {{ outage.check.spec.kind | checkkind() }}

  td.uk-table-shrink.uk-text-nowrap(class='uk-hidden@m')
    span.checkkind {{ outage.check.spec.kind | checkkind() }}

  td.uk-table-shrink.uk-text-nowrap(class='uk-visible@m')
    p {{ outage.started_on | moment("from") }}

  td.uk-table-shrink.uk-text-nowrap(class='uk-visible@m')
    p(v-if='outage.ended_on') {{ lasted(outage) | duration("humanize") }}
    p.uk-text-bold.uk-text-warning(v-else) Ongoing

  td.uk-table-shrink.uk-text-nowrap(class='uk-hidden@m')
    p
      span {{ outage.started_on | moment("from") }}
      span {{ " → " }}
      span(v-if='outage.ended_on') {{ lasted(outage) | duration("humanize") }}
      span.uk-text-bold.uk-text-warning(v-else) Ongoing

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
