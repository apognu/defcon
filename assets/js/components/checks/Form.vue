<template lang="pug">
div
  h2(v-if='new_record') Create a check
  h2(v-else) Edit a check

  .uk-card.uk-card-default.uk-card-body.uk-margin
    h3.uk-card-title Basic settings

    .uk-grid-small(v-if='check || new_record', uk-grid)
      .uk-width-1-3
        label.uk-form-label Check name
        .uk-form-controls
          input.uk-input(
            type='text',
            v-model='check.name',
            @keyup.enter='save()'
          )

      .uk-width-1-3
        label.uk-form-label Group
        .uk-form-controls
          select.uk-select(v-model='check.group')
            option(:value='undefined') -
            option(
              v-for='group in groups',
              :key='group.uuid',
              :value='group.uuid'
            ) {{ group.name }}

      .uk-width-1-3
        label.uk-form-label Alerter
        .uk-form-controls
          select.uk-select(v-model='check.alerter')
            option(:value='undefined') -
            option(
              v-for='alerter in alerters',
              :key='alerter.uuid',
              :value='alerter.uuid'
            ) {{ alerter.name }}

      .uk-width-1-4
        label.uk-form-label Interval
        .uk-form-controls
          input.uk-input(
            type='text',
            v-model='check.interval',
            @keyup.enter='save()'
          )

      .uk-width-1-4
        label.uk-form-label Failing threshold
        .uk-form-controls
          input.uk-input(
            type='text',
            v-model.number='check.failing_threshold',
            @keyup.enter='save()'
          )

      .uk-width-1-4
        label.uk-form-label Passing threshold
        .uk-form-controls
          input.uk-input(
            type='text',
            v-model.number='check.passing_threshold',
            @keyup.enter='save()'
          )

      .uk-width-1-4
        label.uk-form-label Site threshold
        .uk-form-controls
          input.uk-input(
            type='text',
            v-model.number='check.site_threshold',
            @keyup.enter='save()'
          )

  .uk-card.uk-card-default.uk-card-body.uk-margin
    h3.uk-card-title(v-if='new_record') Check specification
    h3.uk-card-title(v-else) {{ check.spec.kind | checkkind() }}

    .uk-grid-small(v-if='new_record', uk-grid)
      .uk-width-1-1
        label.uk-form-label Check type
        .uk-form-controls
          select.uk-select(v-model='check.spec.kind')
            option(v-for='kind in kinds', :key='kind', :value='kind') {{ kind | checkkind() }}

    .uk-margin(v-if='check')
      Http(v-if='check.spec.kind == "http"', :spec='check.spec', ref='spec')
      Dns(v-if='check.spec.kind == "dns"', :spec='check.spec', ref='spec')
      Tcp(v-if='check.spec.kind == "tcp"', :spec='check.spec', ref='spec')
      Udp(v-if='check.spec.kind == "udp"', :spec='check.spec', ref='spec')
      PlayStore(
        v-if='check.spec.kind == "play_store"',
        :spec='check.spec',
        ref='spec'
      )
      AppStore(
        v-if='check.spec.kind == "app_store"',
        :spec='check.spec',
        ref='spec'
      )
      Tls(v-if='check.spec.kind == "tls"', :spec='check.spec', ref='spec')
      Whois(v-if='check.spec.kind == "whois"', :spec='check.spec', ref='spec')
      Ping(v-if='check.spec.kind == "ping"', :spec='check.spec', ref='spec')

  .uk-margin-top
    button.uk-button.uk-button-primary.uk-button-small(@click='save') Save check
</template>

<script>
import axios from 'axios';

import Http from '@/components/checks/forms/Http.vue';
import Dns from '@/components/checks/forms/Dns.vue';
import Tcp from '@/components/checks/forms/Tcp.vue';
import Udp from '@/components/checks/forms/Udp.vue';
import PlayStore from '@/components/checks/forms/PlayStore.vue';
import AppStore from '@/components/checks/forms/AppStore.vue';
import Tls from '@/components/checks/forms/Tls.vue';
import Whois from '@/components/checks/forms/Whois.vue';
import Ping from '@/components/checks/forms/Ping.vue';

export default {
  components: {
    Http,
    Dns,
    Tcp,
    Udp,
    PlayStore,
    AppStore,
    Tls,
    Whois,
    Ping,
  },

  data: () => ({
    check: undefined,
    groups: [],
    alerters: [],
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
      axios.get(`/api/checks/${this.$route.params.uuid}`).then((response) => {
        this.check = response.data;

        if (this.check.group !== undefined) {
          this.check.group = this.check.group.uuid;
        }

        if (this.check.alerter !== undefined) {
          this.check.alerter = this.check.alerter.uuid;
        }
      });
    }

    axios.get('/api/groups').then((response) => {
      this.groups = response.data;
    });

    axios.get('/api/alerters').then((response) => {
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

      delete body.uuid;

      Object.keys(body.spec).forEach((key) => {
        if (!body.spec[key] || body.spec[key] === '') {
          delete body.spec[key];
        }
      });

      if (this.new_record) {
        axios.post('/api/checks', body).then(() => {
          this.$router.push({ name: 'checks' });
        });
      } else {
        axios.put(`/api/checks/${this.$route.params.uuid}`, body).then(() => {
          this.$router.push({
            name: 'checks.view',
            params: { uuid: this.$route.params.uuid },
          });
        });
      }
    },
  },
};
</script>
