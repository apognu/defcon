# DNS

The DNS handler will retrieve DNS records of a specific type, for a specific domain, and check if one of the values matches the specification. Only the following DNS record types are supported:

 * NS
 * MX
 * A
 * AAAA
 * CNAME
 * CAA

## Attributes

| Attribute | Type   | Example         | Description                                                   |
| --------- | ------ | --------------- | ------------------------------------------------------------- |
| `kind`    | string | `"dns"`         | -                                                             |
| `record`  | string | `"A"`           | Type of DNS record to verify                                  |
| `domain`  | string | `"example.com"` | Domain name for which the retrieve the records                |
| `value`   | string | `"1.2.3.4"`     | Value to compare to each retrieved record, must match exactly |

## Configuration

You can change the DNS resolver used to resolve DNS records by using the `DNS_RESOLVER` environment variable when starting the controller and the runners, like so: `DNS_RESOLVER=1.2.3.4`. By default, `1.1.1.1` is used.
