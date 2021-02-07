# TCP connection

This handler will attempt to open a TCP connection on a provided host and port, and fail if the connection is unsuccessful.

## Attributes

| Attribute | Type   | Example           | Description                                  |
| --------- | ------ | ----------------- | -------------------------------------------- |
| `kind`    | string | `"tcp"`           | -                                            |
| `host`    | string | `"93.184.216.34"` | Domain name or IP address of the target host |
| `port`    | int    | 80                | Port on which to open the TCP connection     |
