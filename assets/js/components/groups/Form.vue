<template lang="pug">
div
  h2(v-if='new_record') New group
  template(v-else-if='group')
    p.uk-margin-remove.uk-text-small.uk-text-bolder.uk-text-uppercase Edit group
    h2.uk-margin-remove-top {{ group.name }}

  .uk-form-horizontal(v-if='group || new_record')
    .uk-margin
      label.uk-form-label Group name
      .uk-form-controls
        input.uk-input(
          type='text',
          v-model='group.name',
          @keyup.enter='save()'
        )

    .uk-margin-top
      button.uk-button.uk-button-primary.uk-button-small(@click='save') Save group
</template>

<script>
import axios from 'axios';

export default {
  data: () => ({
    group: undefined,
  }),

  computed: {
    new_record() {
      return this.$route.meta.action === 'new';
    },
  },

  async mounted() {
    if (this.new_record) {
      this.group = {};
    } else {
      axios.get(`/api/groups/${this.$route.params.uuid}`).then((response) => {
        this.group = response.data;
      });
    }
  },

  methods: {
    save() {
      const body = {
        name: this.group.name,
      };

      if (this.new_record) {
        axios.post('/api/groups', body).then(() => {
          this.$router.push({ name: 'groups' });
        });
      } else {
        axios.put(`/api/groups/${this.$route.params.uuid}`, body).then(() => {
          this.$router.push({ name: 'groups' });
        });
      }
    },
  },
};
</script>
