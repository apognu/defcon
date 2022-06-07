# Dead Man Switch

The Dead Man Switch (DMS) handler will trigger an alert if an external service has not "checked in" in some time.

More precisely, a separate HTTP server is spawned on which external service can send GET requests to "check in". These services would usually check in after performing some task successfully (like a backup process, for instance) to let Defcon know the task finished successfully. If a check in is missed, this would indicate the task has failed, triggering an alert.

## Attributes

| Attribute     | Type   | Example           | Description                                                          |
| ------------- | ------ | ----------------- | -------------------------------------------------------------------- |
| `kind`        | string | `"deadmanswitch"` | -                                                                    |
| `stale_after` | string | `"1h"`            | The duration after which to create an outage if no check in happened |

## Configuration

The `DMS_ENABLE` can be used to disable the HTTP server used to receive checkins. Additionally, its listening address (127.0.0.1:8080 by default) can be configured through `DMS_LISTEN`.

To check in, a service needs to perform a GET request at `http://${LISTEN_ADDRESS}/checkin/<check_id>`.
