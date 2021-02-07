# Ping

This handler sends one ICMP echo request to the specified host and reports an error if it is unsuccessful.

> Note that this may require some sort of elevated privilege to be able to run. For example, on Linux, it needs either to be run as `root` (not recommended), or to have the `CAP_NET_RAW` capability.
>
> To set `CAP_NET_RAW`, you can execute the following command on Defcon's binary:
>
> ```shell
> setcap cap_net_raw+ep defcon
> ```

## Attributes

| Attribute | Type   | Example     | Description                                 |
| --------- | ------ | ----------- | ------------------------------------------- |
| `kind`    | string | `"ping"`    | -                                           |
| `host`    | string | `"8.8.8.8"` | Host to which to send the ICMP echo request |
