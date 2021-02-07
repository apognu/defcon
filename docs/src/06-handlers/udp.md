# UDP datagram

The UDP handler will attempt to send a datagram to a host and port, and expect to receive a specific response in return.

## Attributes

| Attribute | Type   | Example          | Description                                                                                    |
| --------- | ------ | ---------------- | ---------------------------------------------------------------------------------------------- |
| `kind`    | string | `"udp"`          | -                                                                                              |
| `host`    | string | `"1.2.3.4"`      | Target host where to send the datagram                                                         |
| `port`    | int    | `10000`          | Port to use as destination                                                                     |
| `message` | string | `"aGVsbG8="`     | Base 64-encoded message to send on the socket                                                  |
| `timeout` | string | `"5s"`           | Timeout before giving up on the response                                                       |
| `content` | string | `"Z29vZGJ5ZQ=="` | Base64-encoded value to expect in the response. This must be a submatch of the actual response |
