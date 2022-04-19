<template lang="pug">
table.uk-table.uk-table-middle.uk-table-divider
  tr(v-for='event in events')
    td.uk-table-shrink
      .bubble.success(v-if='event.status === 0')
      .bubble.error(v-else)

    td(class='uk-hidden@m')
      p.uk-margin-remove {{ event.created_at | moment("from") }}
      p.uk-margin-remove: span.checkkind {{ event.site }}

    td.uk-table-shrink.uk-text-nowrap(class='uk-visible@m')
      p {{ event.created_at | moment("from") }}

    td.uk-table-shrink(class='uk-visible@m')
      span.checkkind {{ event.site }}

    td.uk-text-right {{ event.message }}
</template>

<script>
export default {
  props: {
    events: {
      type: Array,
      required: true,
    },
  },
};
</script>
