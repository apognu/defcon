# Domain expiration

This handler will check Whois databases for the provided domain and will attempt to retrieve the domain's expiration date. The emitted event will be marked as failed if the expiration is within the configured window.

> Not all TLDs expose their domains' expiration date in the Whois response, this handler will only work for those that do.

## Attributes

| Attribute   | Type   | Example         | Description                                                                       |
| ----------- | ------ | --------------- | --------------------------------------------------------------------------------- |
| `kind`      | string | `"domain"`      | -                                                                                 |
| `domain`    | string | `"example.com"` | The base domain name to check for                                                 |
| `window`    | string | `"60d"`         | Period of time within which the handler should fail                               |
| `attribute` | string | `"expiry date"` | Whois attribute to use as the expiration date. Defaults to `registry expiry date` |
