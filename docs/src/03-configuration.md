# Configuration

The controller has a few configuration knobs you can tweak to adjust its overall behavior, they are described in this document. Most of them are optional and default to maybe-no-so sensible values. All configuration is applied through environment variables.

## Handler configuration

| Environment variable | Required | Default value  | Description                                            |
| -------------------- | -------- | -------------- | ------------------------------------------------------ |
| `RUST_LOG`           |          | defcon=info    |                                                        |
| `DSN`                | Yes      |                | Connection string to the MySQL database                |
| `PUBLIC_KEY`         | Yes      |                | Path to an PEM-encoded ECDSA public key                |
| `API_ENABLE`         |          | 1              | Enable or disable the API process                      |
| `API_LISTEN`         |          | 127.0.0.1:8000 | Set the listen address and port of the API process     |
| `WEB_ENABLE`         |          | 0              | Enable or disable the Web administration interface     |
| `HANDLER_ENABLE`     |          | 1              | Enable or disable the handler process                  |
| `HANDLER_INTERVAL`   |          | 1s             | Interval between handler loop iterations               |
| `HANDLER_SPREAD`     |          | 0s             | Maximum random delay applied when a check needs to run |
| `CLEANER_ENABLE`     |          | 0              | Enable or disable the cleaner process                  |
| `CLEANER_INTERVAL`   |          | 10m            | Interval between cleaner loop iterations               |
| `CLEANER_THRESHOLD`  |          | 1y             | Period of time after which to delete stale objects     |
| `ALERTER_DEFAULT`    |          |                | Alerter to create checks with, if unspecified          |
| `ALERTER_FALLBACK`   |          |                | Alerter to be called when none is set on a check       |

### `RUST_LOG`

This allows for controlling the log level for each individual dependency. Defcon uses the `defcon` identifier, and default to `info`, which will mainly print errors, API access logs and when outages are created and resolved.

### `DSN`

This should be a full connection string (with options) to the database Defcon is to use, and start with `mysql://`, as this is the only database supported by Defcon. This is an example of a valid `DSN`.

```
mysql://user:password@host:3306/defcon
```

### `PUBLIC_KEY`

This option should contain the path to an existing PEM-encoded ECDSA public key. The following command can generate a compatible public key for usage with Defcon's controller:

```shell
$ openssl ecparam -genkey -name prime256v1 -noout | openssl pkcs8 -topk8 -nocrypt -out defcon-private.pem
$ openssl ec -in defcon-private.pem -pubout -out defcon-public.pem
```

### `API_ENABLE`

A value of `0` or `1` respectively disables and enables the API process bundled within Defcon.

### `API_LISTEN`

A string representing an IP address and port on which the API process will bind its process. By default, the API is only reachable by the local host on port 8000.

### `WEB_ENABLE`

A value of `0` or `1` respectively disables and enabled the Web administration interface to manage and visualization Defcon's operations.

### `HANDLER_ENABLE`

A value of `0` here disables check handling on the controller. If it is disabled, no check will run on this node. This is particularly useful if all your checks are configured to run on off-site runners and you would prefer to use the controller only as Defcon's control plane.

### `HANDLER_INTERVAL`

If the handler process is enabled, this setting defines at which interval we should try to determine if checks need to be run. Accepts human-readable durations (such as `1s` or `5m`), defaults to `1s` and cannot be smaller than one second.

### `HANDLER_SPREAD`

Maximum amount of time for the controller to wait before executing a stale check. This can be useful to prevent all checks running at the exact same time. When a check needs executing, if `HANDLER_SPREAD` is set to `5s`, Defcon will wait for a random duration between `0s` and `5s` before executing the related handler.

### `CLEANER_ENABLE`

Use `1` here if you wish to enable the cleaner process. The cleaner process is used to delete old items (events, site outages and confirmed outages) from the database. Only resolved outages are elligible to be cleaned.

### `CLEANER_INTERVAL`

This option defines the interval at which Defcon will check for database items to be deleted from the database. This is a maintenance operation and does not need to run as often as the handler process.

### `CLEANER_THRESHOLD`

How old should an outage be to be elligible for deletion? Here, a value of `6 months` will delete all resolved outages, site outages and events that are at least six months old.
