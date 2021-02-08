# Defcon

Defcon is a tool allows you to define and periodically run monitoring checks (also called _uptime checks_) against external services.

You can find more extensive documentation in the [user manual](https://apognu.github.io/defcon/).

## How to run

### Requirements

Defcon requires the following infrastructure to be run:

 * At least one Linux server to run it on
 * A MySQL database

Until clearly stated, the database schema is subject to breaking changes, and migrations will not be used.

### Configuration

Some of Defcon's default behavior can be customized through environment variables. You can find all supported configuration variables in the table below:

Configuration options for the controller can be found in the [user manual](https://apognu.github.io/defcon/03-configuration.html).

### Let's go!

```sh
$ DSN=mysql://defcon:password@mysql.host/defcon?ssl-mode=DISABLED \
  PUBLIC_KEY=/path/to/public/key.pem \
  defcon
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
  "sites": ["eu-1", "eu-2"],
  "interval": "1m",
  "site_threshold": 2,
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

Defcon comes with four components:

 * An **API** process, used as our control plane
 * A **cleaner** process, optionally used to delete resolved outages and events
 * A **handler** process, in charge of actually running the cruft of Defcon
 * An independent **runner** that pulls elligible checks to be run on a remote machine

The **handler** process, every `HANDLER_INTERVAL`, will look at all `enabled` checks and, depending on the timestamp for their last emitted event, determine which one should be run (depending on their respective `interval`s).

If a checks returns an error unrelated to the monitored service (`permission denied` to open local raw socket, for example), no event is emitted an no outage is created. Moreover, the next run for the check will be delayed by `interval` to prevent spam.

When an outage is confirmed, an optional `alerter` is called, with details attached, to export the outage and related objects to a Slack channel (through a webhook) or to a generic webhook URL.

### Multi-site monitoring

On top of the main controller, defcon comes with a runner that is able to be run on other machines to help monitor services from multiple locations. Each check is created with a list of locations where it should be run as well as a threshold of failing `sites` (read: _locations_) above which the service will be considered as globally failing. We then have two kinds of outages:

 * Site-wide outages are triggered when a check exceeds its `failing_threshold` on a specific site. Alerts are not sent for this kind of outage.
 * Global outages are triggered when the number of site-wide outages for a check exceeds `site_threshold`. It is resolved when it falls under that threshold. Alerts are sent for these.

Sites are only represented as tag values in the `sites` attribute on checks that defines on which sites a check should run. You should configure the runners with the tag value for their site. There should only be one runner using a specific tag value. The controller has a special tag value of `@controller`. Other tag values should conform to `[a-z0-9-]+`.

Runners are authenticated through a common shared private key, used to sign token appended to the requests to the controller.

A runner will periodically ask the controller for all checks that are due for running and locally launch the handlers for those checks. When one of those checks completes, it reports its status back to the controller.

In order to launch a runner, the following command can be performed:

```shell
$ PRIVATE_KEY=./defcon-private.pem \
  CONTROLLER_URL=https://controller.example.com \
  SITE=eu-west-1 \
  POLL_INTERVAL=30s \
  defcon-runner
```

## Handlers

| Check name        | Internal ID  | Description                                                                    |
| ----------------- | ------------ | ------------------------------------------------------------------------------ |
| iOS app           | `app_store`  | Verify if an iOS app can be found on the App Store                             |
| DNS record        | `dns`        | Verify the value for a domain record (`NS`, `MX`, `A`, `AAAA`, `CNAME`, `CAA`) |
| HTTP request      | `http`       | Verify the response to an HTTP GET request                                     |
| ICMP echo request | `ping`       | Verify if a host can be pinged                                                 |
| Android app       | `play_store` | Verify if an Android app can be found on the Play Store                        |
| TCP connection    | `tcp`        | Verify if a host is reachable through a TCP port                               |
| TLS expiration    | `tls`        | Verify the expiration date for a TLS certificate                               |
| UDP datagram      | `udp`        | Verify the response from a host on a UDP port                                  |
| Domain expiration | `whois`      | Verify the expiration date for a domain registration                           |

You can find detailed explanations about how to configure each of those handlers in the [user manual](https://apognu.github.io/defcon/).

## API

Defcon exposes an **unauthenticated** API used to manipulate and retrieve the data it uses internally. The available endpoints are detailed in the table below:

| Method | Endpoint                                                               | Description                                          |
| ------ | ---------------------------------------------------------------------- | ---------------------------------------------------- |
| GET    | /api/-/health                                                          | Health endpoint                                      |
| GET    | /api/checks                                                            | List all enabled checks                              |
| GET    | /api/check?all=true                                                    | List all defined checks                              |
| GET    | /api/checks/`uuid`                                                     | Get information on a specific check                  |
| POST   | /api/checks                                                            | Create a new check                                   |
| PUT    | /api/checks/`uuid`                                                     | Update a check                                       |
| PATCH  | /api/checks/`uuid`                                                     | Update a check, with only attributes to update       |
| DELETE | /api/checks/`uuid`                                                     | Disables a check                                     |
| GET    | /api/sites/outages                                                     | List all active site outages, with the related check |
| GET    | /api/sites/outages?from=`YYYY-MM-DDTHH:MM:SS`&to=`YYYY-MM-DDTHH:MM:SS` | List all site outages during a time period           |
| GET    | /api/sites/outages/`uuid`                                              | Get information on a specific site outage            |
| GET    | /api/sites/outages/`uuid`/events                                       | Get all events related to a site outage              |
| GET    | /api/outages                                                           | List all active outages, with the related check      |
| GET    | /api/outages?from=`YYYY-MM-DDTHH:MM:SS`&to=`YYYY-MM-DDTHH:MM:SS`       | List all outages during a time period                |
| PUT    | /api/outages/`uuid`/comment                                            | Add a comment to an outage                           |
| GET    | /api/alerters                                                          | Get all configured alerters                          |
| GET    | /api/alerters/`uuid`                                                   | Get information on a specific alerter                |
| POST   | /api/alerters                                                          | Create a new alerter                                 |
| PUT    | /api/alerters/`uuid`                                                   | Update an existing alerter                           |

Did you pay attention to the fact that this API is **NOT AUTHENTICATED** and should therefore be used behind some kind of reverse proxy that will add some semblance of security to it?

## What's next?

 * More check types (ideas and PRs are welcome)?
 * Statistics API
 * More extensive, step-by-step documentation
 * Site registration and specific runner authentication
