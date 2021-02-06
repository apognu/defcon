# Defcon

Defcon is a tool for monitoring external services for specific failure scenarii. You can see it as a lightweight Nagios focused on watching over networked services. This kind of tool is sometimes calls _uptime monitoring services_.

A common example use case of what you can do with Defcon is periodically perform an HTTP request to an API endpoint and verify that the response status code is `200 OK` and the content contains the words `ready`, sending a message on a Slack channel whenever the check fails three times in a row.

> This documentation is still in the process of being written, it is far from complete and could not bring you all the information you need.

## Concepts

### Checks

The main concept in Defcon is that of a **check**. A check is a definition for an external service, including how to monitor it, how to detect issues, when to consider it as failing and what do do when it is.

Each check includes what is called a **handler specification** (or `spec`), that describes how this service is to be monitored. This specification must be one of the supported handlers, as described in this section.

### Alerters

Each check can optionally trigger an alert when an outage is confirmed by its handler. All alerters in Defcon use `webhooks` to transmit information about the failing check to a HTTP server (outside Defcon's scope) that will handle the actual notification handling.

### Site

A site is a distinct location (read, _server_), where checks can be run from. This allows for monitoring services from different locations concurrently to help avoid detection issues caused by hardware failing, network partitions or transient problems.

Each check can be set to run on one of more different sites, with a configurable number of failing sites needed for an outage to be confirmed.
