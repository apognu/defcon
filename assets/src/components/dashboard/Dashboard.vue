<template lang="pug">
div
  h2 Dashboard

  Status
  Timeline(:showUptime='true')
</template>

<script>
import Status from '~/components/dashboard/Status.vue';
import Timeline from '~/components/dashboard/Timeline.vue';

export default {
  components: {
    Status,
    Timeline,
  },
};
</script>
