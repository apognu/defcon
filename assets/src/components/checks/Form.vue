<template lang="pug">
div(v-if='check')
  h2(v-if='new_record') New check
  template(v-else-if='check')
    p.uk-margin-remove.uk-text-small.uk-text-bolder.uk-text-uppercase Edit check
    h2.uk-margin-remove-top {{ check.name }}

  .uk-card.uk-card-default.uk-card-body.uk-margin
    h3.uk-card-title Basic settings

    div(v-if='check || new_record')
      .uk-grid-small(uk-grid, class='uk-child-width-1-3@m')
        div
          label.uk-form-label Check name
          .uk-form-controls
            input.uk-input(
              type='text',
              v-model='check.name',
              @keyup.enter='save()'
            )

        div
          label.uk-form-label Group
          .uk-form-controls
            select.uk-select(v-model='check.group')
              option(:value='undefined') -
              option(
                v-for='group in groups',
                :key='group.uuid',
                :value='group.uuid'
              ) {{ group.name }}

        div
          label.uk-form-label Alerter
          .uk-form-controls
            select.uk-select(v-model='check.alerter')
              option(:value='undefined') -
              option(
                v-for='alerter in alerters',
                :key='alerter.uuid',
                :value='alerter.uuid'
              ) {{ alerter.name }}

      .uk-grid-small(uk-grid, class='uk-child-width-1-4@m uk-child-width-1-2@s')
        div
          label.uk-form-label Interval
          .uk-form-controls
            input.uk-input(
              type='text',
              v-model='check.interval',
              @keyup.enter='save()'
            )

        div
          label.uk-form-label Failing threshold
          .uk-form-controls
            input.uk-input(
              type='text',
              v-model.number='check.failing_threshold',
              @keyup.enter='save()'
            )

        div
          label.uk-form-label Passing threshold
          .uk-form-controls
            input.uk-input(
              type='text',
              v-model.number='check.passing_threshold',
              @keyup.enter='save()'
            )

        div
          label.uk-form-label Site threshold
          .uk-form-controls
            input.uk-input(
              type='text',
              v-model.number='check.site_threshold',
              @keyup.enter='save()'
            )

      .uk-grid-small(uk-grid, class='uk-child-width-1-1@m uk-child-width-1-1@s')
        div
          label.uk-form-label Site labels
          .uk-form-controls
            vue-tags-input(v-model='site', :tags='sites', @tags-changed='(data) => sites = data')

  .uk-card.uk-card-default.uk-card-body.uk-margin
    h3.uk-card-title(v-if='new_record') Check specification
    h3.uk-card-title(v-else) {{ $filters.checkkind(check.spec.kind) }}

    .uk-grid-small(v-if='new_record', uk-grid)
      .uk-width-1-1
        label.uk-form-label Check type
        .uk-form-controls
          select.uk-select(v-model='check.spec.kind')
            option(v-for='kind in kinds', :key='kind', :value='kind') {{ $filters.checkkind(kind) }}

    .uk-margin(v-if='check')
      Http(
        v-if='check.spec.kind == "http"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      Dns(
        v-if='check.spec.kind == "dns"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      Tcp(
        v-if='check.spec.kind == "tcp"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      Udp(
        v-if='check.spec.kind == "udp"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      PlayStore(
        v-if='check.spec.kind == "play_store"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      AppStore(
        v-if='check.spec.kind == "app_store"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      Tls(
        v-if='check.spec.kind == "tls"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      Whois(
        v-if='check.spec.kind == "whois"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      Ping(
        v-if='check.spec.kind == "ping"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      Python(
        v-if='check.spec.kind == "python"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )
      DeadManSwitch(
        v-if='check.spec.kind == "deadmanswitch"',
        :spec='check.spec',
        ref='spec',
        @enter='save()'
      )

  .uk-margin-top
    button.uk-button.uk-button-primary.uk-button-small(@click='save') Save check
</template>

<script>
import VueTagsInput from '@sipec/vue3-tags-input';

import Http from '~/components/checks/forms/Http.vue';
import Dns from '~/components/checks/forms/Dns.vue';
import Tcp from '~/components/checks/forms/Tcp.vue';
import Udp from '~/components/checks/forms/Udp.vue';
import PlayStore from '~/components/checks/forms/PlayStore.vue';
import AppStore from '~/components/checks/forms/AppStore.vue';
import Tls from '~/components/checks/forms/Tls.vue';
import Whois from '~/components/checks/forms/Whois.vue';
import Ping from '~/components/checks/forms/Ping.vue';
import Python from '~/components/checks/forms/Python.vue';
import DeadManSwitch from '~/components/checks/forms/DeadManSwitch.vue';

export default {
  components: {
    VueTagsInput,
    Http,
    Dns,
    Tcp,
    Udp,
    PlayStore,
    AppStore,
    Tls,
    Whois,
    Ping,
    Python,
    DeadManSwitch,
  },

  inject: ['$http', '$filters'],

  data: () => ({
    check: undefined,
    groups: [],
    alerters: [],
    sites: [{ text: '@controller' }],
    site: '',
    kinds: [
      'http',
      'tls',
      'dns',
      'tcp',
      'udp',
      'ping',
      'app_store',
      'play_store',
      'whois',
      'deadmanswitch',
    ],
  }),

  computed: {
    new_record() {
      return this.$route.meta.action === 'new';
    },
  },

  async mounted() {
    if (this.new_record) {
      this.check = {
        interval: '5m',
        failing_threshold: 3,
        passing_threshold: 3,
        site_threshold: 1,
        spec: {},
      };
    } else {
      this.$http().get(`/api/checks/${this.$route.params.uuid}`).then((response) => {
        this.check = response.data;

        this.sites = this.check.sites.map((site) => ({ text: site }));

        if (this.check.group !== undefined) {
          this.check.group = this.check.group.uuid;
        }

        if (this.check.alerter !== undefined) {
          this.check.alerter = this.check.alerter.uuid;
        }
      });
    }

    this.$http().get('/api/groups').then((response) => {
      this.groups = response.data;
    });

    this.$http().get('/api/alerters').then((response) => {
      this.alerters = response.data;
    });
  },

  methods: {
    save() {
      const spec = this.$refs.spec.serialize();
      const body = { ...this.check, spec };

      if (body.group === '') {
        body.group = null;
      }

      if (body.alerter === '') {
        body.alerter = null;
      }

      body.sites = this.sites.map((site) => site.text);

      delete body.uuid;

      Object.keys(body.spec).forEach((key) => {
        if (!body.spec[key] || body.spec[key] === '') {
          delete body.spec[key];
        }
      });

      if (this.new_record) {
        this.$http()
          .post('/api/checks', body)
          .then(() => {
            this.$router.push({ name: 'checks' });
          })
          .catch((e) => {
            this.$helpers.error(`${e.message}: ${e.response.data.details}`);
          });
      } else {
        this.$http()
          .put(`/api/checks/${this.$route.params.uuid}`, body)
          .then(() => {
            this.$router.push({
              name: 'checks.view',
              params: { uuid: this.$route.params.uuid },
            });
          })
          .catch((e) => {
            this.$helpers.error(`${e.message}: ${e.response.data.details}`);
          });
      }
    },
  },
};
</script>

<style lang="scss">
.vue-tags-input {
  max-width: none !important;

  .ti-tag {
    padding: 6px 8px !important;
    background: #1e87f0 !important;
  }
}
</style>
