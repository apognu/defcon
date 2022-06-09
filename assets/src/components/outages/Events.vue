<template lang="pug">
table.uk-table.uk-table-middle.uk-table-divider
  tr(v-for='event in events')
    td.uk-table-shrink
      .bubble.success(v-if='event.status === 0')
      .bubble.error(v-else)

    td(class='uk-hidden@m')
      p.uk-margin-remove {{ $helpers.ago(event.created_at) }}
      p.uk-margin-remove: span.checkkind {{ event.site }}

    td.uk-table-shrink.uk-text-nowrap(class='uk-visible@m')
      p(:uk-tooltip='`title: ${$helpers.datetime(event.created_at)}`') {{ $helpers.ago(event.created_at) }}

    td.uk-table-shrink(class='uk-visible@m')
      span.checkkind {{ event.site }}

    td.uk-text-right {{ event.message }}
</template>

<script>
export default {
  inject: ['$helpers'],

  props: {
    events: {
      type: Array,
      required: true,
    },
  },
};
</script>
