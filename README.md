# Defcon

Defcon is a tool allows you to define and periodically run monitoring checks (also called _uptime checks_) against external services.

## How to run

### Requirements

Defcon requires the following infrastructure to be run:

 * A server to run it on (only tested on Linux as of now)
 * A MySQL database

### Configuration

Some of Defcon's default behavior can be customized through environment variables. You can find all supported configuration variables in the table below:

| Environment variable | Required | Default value | Description                                            |
| -------------------- | -------- | ------------- | ------------------------------------------------------ |
| `RUST_LOG`           | false    | defcon=info   |                                                        |
| `DSN`                | true     |               | Connection string to the MySQL database                |
| `API_ENABLE`         | false    | 1             | Enable or disable the API process                      |
| `API_PORT`           | false    | 8000          | Set the listen port of the API process                 |
| `HANDLER_ENABLE`     | false    | 1             | Enable or disable the handler process                  |
| `HANDLER_INTERVAL`   | false    | 1s            | Interval between handler loop iterations               |
| `HANDLER_SPREAD`     | false    | 0s            | Maximum random delay applied when a check needs to run |
| `CLEANER_ENABLE`     | false    | 0             | Enable or disable the cleaner process                  |
| `CLEANER_INTERVAL`   | false    | 10m           | Interval between cleaner loop iterations               |
| `CLEANER_THRESHOLD`  | false    | 1y            | Period of time after which to delete stale objects     |

### Let's go!

```shell
$ DSN=mysql://defcon:password@mysql.host/defcon?ssl-mode=DISABLED defcon
INFO[2021-01-30T00:19:39.576+0000] started API server on port 8000
INFO[2021-01-30T00:19:39.576+0000] started handler loop
```

## Concepts

Defcon allows you to create **checks** used to describe external services to be monitored`, `how often it should be monitored, and some options for state change thresholds. The actual check that is performed is described in a check's **spec**.

This spec is going to be given to the **handler** that is able to perform the check and determine if it succeeds or fails. This handler will produce an **event** describing the status for the check, at a given time, and potentially include some details about the result.

A check is defined as so (here, for an HTTP request check):

```json
{
  "name": "ACME corporate website",
  "uuid": "50a5c57f-6971-446a-b9a2-42cb7c7b5427",
  "alerter": "df2dcc77-00c1-4dc1-a8a3-6ba0bc64d486",
  "enabled": true,
  "interval": "1m",
  "passing_threshold": 3,
  "failing_threshold": 2,
  "silent": false,
  "spec": {
    "kind": "http",
    "code": null,
    "content": "© ACME Inc. 2021",
    "digest": "04436440f3615902838b18...b16c4d848d7408",
    "headers": {
      "accept": "application/json"
    },
    "url": "https://example.com/health"
  }
}
```

When a check fails, an **outage** is created, and kept until such time that the check passes again.

Defcon comes with three components:

 * An **API** process, used as our control plane
 * A **cleaner** process, optionally used to delete resolved outages and events
 * A **handler** process, in charge of actually running the cruft of Defcon

The **handler** process, every `HANDLER_INTERVAL`, will look at all `enabled` checks and, depending on the timestamp for their last emitted event, determine which one should be run (depending on their respective `interval`s).

If a checks returns an error unrelated to the monitored service (`permission denied` to open local raw socket, for example), no event is emitted an no outage is created. Moreover, the next run for the check will be delayed by `interval` to prevent spam.

When an outage is confirmed, an optional `alerter` is called, with details attached, to export the outage and related objects to a Slack channel (through a webhook) or to a generic webhook URL.

## Checks

| Check name          | Internal ID  | Description                                                                    |
| ------------------- | ------------ | ------------------------------------------------------------------------------ |
| iOS app             | `app_store`  | Verify if an iOS app can be found on the App Store                             |
| DNS record          | `dns`        | Verify the value for a domain record (`NS`, `MX`, `A`, `AAAA`, `CNAME`, `CAA`) |
| HTTP request        | `http`       | Verify the response to an HTTP GET request                                     |
| ICMP echo request ¹ | `ping`       | Verify if a host can be pinged                                                 |
| Android app         | `play_store` | Verify if an Android app can be found on the Play Store                        |
| TCP connection      | `tcp`        | Verify if a host is reachable through a TCP port                               |
| TLS expiration      | `tls`        | Verify the expiration date for a TLS certificate                               |
| UDP datagram ²      | `udp`        | Verify the response from a host on a UDP port                                  |
| Domain expiration ³ | `whois`      | Verify the expiration date for a domain registration                           |

[¹]: Might require proper permission to open raw sockets. On Linux, for example, `CAP_NET_RAW`.

[²]: Local port is chosen randomly, which might require firewall configuration. `message` and `content` should be given as base64-encoded strings.

[³]: Domain registration expiration is not reported through Whois by all TLDs.

You can find example schemas for all these specs (to be used in the API, described in the next section) in the `examples/schemas` directory.

## API

Defcon exposes an **unauthenticated** API used to manipulate and retrieve the data it uses internally. The available endpoints are detailed in the table below:

| Method | Endpoint                                        | Description                                     |
| ------ | ----------------------------------------------- | ----------------------------------------------- |
| GET    | /api/-/health                                   | Health endpoint                                 |
| GET    | /api/checks                                     | List all enabled checks                         |
| GET    | /api/check?all=true                             | List all defined checks                         |
| GET    | /api/checks/`uuid`                              | Get information on a specific check             |
| POST   | /api/checks                                     | Create a new check                              |
| PUT    | /api/checks/`uuid`                              | Update a check                                  |
| PATCH  | /api/checks/`uuid`                              | Update a check, with only attributes to update  |
| DELETE | /api/checks/`uuid`                              | Disables a check                                |
| GET    | /api/outages                                    | List all active outages, with the related check |
| GET    | /api/outages?from=`YYYY-MM-DD`&end=`YYYY-MM-DD` | List all outages during a time period           |
| GET    | /api/outages/`uuid`                             | Get information on a specific outage            |
| GET    | /api/outages/`uuid`/events                      | Get all events related to an outage             |
| PUT    | /api/outages/`uuid`/comment                     | Add a comment to an outage                      |
| GET    | /api/alerters                                   | Get all configured alerters                     |
| GET    | /api/alerters/`uuid`                            | Get information on a specific alerter           |
| POST   | /api/alerters                                   | Create a new alerter                            |
| PUT    | /api/alerters/`uuid`                            | Update an existing alerter                      |

Did you pay attention to the fact that this API is **NOT AUTHENTICATED** and should therefore be used behind some kind of reverse proxy that will add some semblance of security to it?

## What's next?

 * More check types (ideas and PRs are welcome)?
 * Statistics API
 * More extensive, step-by-step documentation
