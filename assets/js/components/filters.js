import Vue from 'vue';

Vue.filter('checkkind', (value) => {
  switch (value) {
    case 'app_store': return 'App Store';
    case 'dns': return 'DNS';
    case 'http': return 'HTTP';
    case 'ping': return 'Ping';
    case 'play_store': return 'Google Play';
    case 'tcp': return 'TCP';
    case 'tls': return 'TLS';
    case 'udp': return 'UDP';
    case 'whois': return 'Domain';
    case 'deadmanswitch': return 'Dead Man Switch';
    default: return 'Unknown';
  }
});

Vue.filter('alerterkind', (value) => {
  switch (value) {
    case 'webhook': return 'Webhook';
    case 'slack': return 'Slack';
    default: return 'Unknown';
  }
});
