import { helpers } from '@vuelidate/validators';

const _json = (value) => {
  try {
    JSON.parse(value);

    return true;
  } catch {
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

export const dnsRecordType = (value) => !helpers.req(value) || helpers.withMessage('Must be a valid DNS record types', _dnsRecordType(value));
export const json = (value) => !helpers.req(value) || helpers.withMessage('Must a valid JSON value', _json(value));

export const duration = helpers.withMessage(
  'This must be a human-readable duration, such as "10m 30s"',
  helpers.regex(/^([0-9]+(s|sec|second|seconds|m|min|minute|minutes|h|hr|hour|hours|d|day|days|w|week|weeks|M|month|months|y|year|years) *)*$/),
);
