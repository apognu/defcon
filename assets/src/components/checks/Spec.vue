<template lang="pug">
.uk-card.uk-card-default
  .uk-card-header
    h3.uk-card-title {{ $filters.checkkind(check.spec.kind) }}

  .uk-card-body
    div(uk-grid, class='uk-child-width-1-2@s uk-child-width-1-4@m')
      Attribute(label='Interval', :uk-tooltip='check.interval') {{ check.interval }}
      Attribute(label='Failing threshold') {{ check.failing_threshold }}
      Attribute(label='Passing threshold') {{ check.passing_threshold }}
      Attribute(label='Site threshold') {{ check.site_threshold }}

    hr

    Http(v-if='check.spec.kind == "http"', :spec='check.spec')
    Dns(v-if='check.spec.kind == "dns"', :spec='check.spec')
    Tcp(v-if='check.spec.kind == "tcp"', :spec='check.spec')
    Udp(v-if='check.spec.kind == "udp"', :spec='check.spec')
    PlayStore(v-if='check.spec.kind == "play_store"', :spec='check.spec')
    AppStore(v-if='check.spec.kind == "app_store"', :spec='check.spec')
    Tls(v-if='check.spec.kind == "tls"', :spec='check.spec')
    Whois(v-if='check.spec.kind == "domain"', :spec='check.spec')
    Ping(v-if='check.spec.kind == "ping"', :spec='check.spec')
    Python(v-if='check.spec.kind == "python"', :spec='check.spec')
    DeadManSwitch(v-if='check.spec.kind == "deadmanswitch"', :spec='check.spec')
</template>

<script>
import Attribute from '~/components/checks/Attribute.vue';
import Http from '~/components/checks/specs/Http.vue';
import Dns from '~/components/checks/specs/Dns.vue';
import Tcp from '~/components/checks/specs/Tcp.vue';
import Udp from '~/components/checks/specs/Udp.vue';
import PlayStore from '~/components/checks/specs/PlayStore.vue';
import AppStore from '~/components/checks/specs/AppStore.vue';
import Tls from '~/components/checks/specs/Tls.vue';
import Whois from '~/components/checks/specs/Whois.vue';
import Ping from '~/components/checks/specs/Ping.vue';
import Python from '~/components/checks/specs/Python.vue';
import DeadManSwitch from '~/components/checks/specs/DeadManSwitch.vue';

export default {
  components: {
    Attribute,
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

  inject: ['$filters', '$helpers'],

  props: {
    check: {
      type: Object,
      default: undefined,
    },
  },
};
</script>
