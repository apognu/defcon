import Vue from 'vue';

Vue.filter('checkkind', (value) => {
  switch (value) {
    case 'app_store': return 'App Store presence';
    case 'dns': return 'DNS record';
    case 'http': return 'HTTP request';
    case 'ping': return 'ICMP Echo request';
    case 'play_store': return 'Play Store presence';
    case 'tcp': return 'TCP connection';
    case 'tls': return 'TLS certificate expiration';
    case 'udp': return 'UDP connection';
    case 'whois': return 'Domain expiration';
    default: return 'Unknown check';
  }
});

Vue.filter('alerterkind', (value) => {
  switch (value) {
    case 'webhook': return 'Plain webhook';
    case 'slack': return 'Slack incoming webhook';
    default: return 'Unknown alerter';
  }
});
