# Checks

A check is used to describe an external service to be monitored. Among other things, it allows to specify some metadata about the check, how and where the check should be run, the actual handler configuration to use and conditions for confirming outages.

## Metadata

| Attribute        | Type   | Example value                            | Description                                                               |
| ---------------- | ------ | ---------------------------------------- | ------------------------------------------------------------------------- |
| `name`           | string | `"acme-public-site"`                     | A human-friendly name used in logs and alerters                           |
| `alerter`        | UUID   | `"19b9eb20-3e3e-46d5-801f-a912e159913c"` | Alerter to be triggered when an outage is created                         |
| `enabled`        | bool   | `true`                                   | When disabled, a check will not run                                       |
| `on_status_page` | bool   | `false`                                  | When enabled, the check will appear on the public static page, if enabled |
| `silent`         | bool   | `false`                                  | When silent, a check will not trigger its alerter                         |
| `group`          | string | `"9b77035c-218e-4d32-bcd7-4a015f7ee147"` | Put the check into a pre-existing group                                   |

## Run and error condition

| Attribute           | Type   | Example value      | Description                                                      |
| ------------------- | ------ | ------------------ | ---------------------------------------------------------------- |
| `sites`             | [int]  | `["us-1", "eu-1"]` | List of sites where this check should run                        |
| `interval`          | string | `"10s"`            | Interval of time between subsequent runs                         |
| `site_threshold`    | int    | 2                  | Number of sites that have to fail to confirm an outage           |
| `failing_threshold` | int    | 3                  | Number of successive fails required to mark a site as failing    |
| `passing_threshold` | int    | 3                  | Number of successive passes required to mark a site as recovered |

> **Note:** if a check is to run on the controller, as well as another site, the controller's identifier should be given explicitely, e.g. `"sites": ["@controller", "eu-1"]`.

## Handler specification

Each check needs one more attribute, `spec`, detailed in the next section, where the handler specification is configured.
