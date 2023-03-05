import { helpers } from '@vuelidate/validators';

const _json = (value) => {
  try {
    JSON.parse(value);

    return true;
  } catch (_) {
    return false;
  }
};

const DNS_RECORD_TYPES = [
  'SOA',
  'NS',
  'A',
  'AAAA',
  'CNAME',
  'DNAME',
  'MX',
  'SRV',
  'TXT',
  'PTR',
  'CAA',
];

const _dnsRecordType = (value) => DNS_RECORD_TYPES.includes(value);

export const dnsRecordType = {
  $validator: (value) => !helpers.req(value) || _dnsRecordType(value),
  $message: 'Must be a valid DNS record types',
};

export const json = {
  $message: 'Must a valid JSON value',
  $validator: (value) => !helpers.req(value) || _json(value),
};

export const duration = {
  $validator: (value) => !helpers.req(value) || helpers.regex(/^([0-9]+(s|sec|second|seconds|m|min|minute|minutes|h|hr|hour|hours|d|day|days|w|week|weeks|M|month|months|y|year|years) *)*$/),
  $message: 'This must be a human-readable duration, such as "10m 30s"',
};
