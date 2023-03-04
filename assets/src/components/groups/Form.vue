<template lang="pug">
div(v-if='group')
  h2(v-if='new_record') New group
  template(v-else-if='group')
    p.uk-margin-remove.uk-text-small.uk-text-bolder.uk-text-uppercase Edit group
    h2.uk-margin-remove-top {{ group.name }}

  .uk-form-horizontal
    .uk-margin
      label.uk-form-label Group name
      .uk-form-controls
        Field(:model='v$.group.name')
          input.uk-input(
            type='text',
            v-model='v$.group.name.$model',
            @keyup.enter='save()'
          )

    .uk-margin-top
      button.uk-button.uk-button-primary.uk-button-small(@click='save') Save group
</template>

<script>
import { useVuelidate } from '@vuelidate/core';
import { required } from '@vuelidate/validators';

import Field from '~/components/partials/Field.vue';

export default {
  inject: ['$http'],

  setup: () => ({
    v$: useVuelidate(),
  }),

  components: {
    Field,
  },

  data: () => ({
    group: undefined,
  }),

  validations: () => ({
    group: {
      name: { required },
    },
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
      this.$http().get(`/api/groups/${this.$route.params.uuid}`).then((response) => {
        this.group = response.data;
      });
    }
  },

  methods: {
    save() {
      this.v$.$validate();

      if (!this.v$.$error()) {
        const body = {
          name: this.group.name,
        };

        if (this.new_record) {
          this.$http()
            .post('/api/groups', body)
            .then(() => {
              this.$router.push({ name: 'groups' });
            });
        } else {
          this.$http()
            .put(`/api/groups/${this.$route.params.uuid}`, body)
            .then(() => {
              this.$router.push({ name: 'groups' });
            });
        }
      }
    },
  },
};
</script>
