# TLS expiration

This handler can retrieve TLS certificates for a website and fail if its expiration date falls within a configurable window of time. This can help detect issues in your renewal processes and be used as a last resort reminder if you still do it manually.

## Attributes

| Attribute | Type   | Example         | Description                                                   |
| --------- | ------ | --------------- | ------------------------------------------------------------- |
| `kind`    | string | `"tls"`         | -                                                             |
| `domain`  | string | `"example.com"` | Domain to retrieve the certificate for                        |
| `window`  | string | `"15d"`         | Period of time before the expiration date to trigger an alert |
