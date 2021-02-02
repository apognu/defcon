# Defcon runner

Allows to offload actual handler operation to another instance and multisite checks.

## Check

A check can be defined with a list of sites on which the check must be run. By default, a check would be run on **[the controller/all sites]**. A special value of `@controller` would represent the controller node.

A new attribute would be used to indicate how many sites must fail/succeed in order for the check to change state (e.g. **[2 out of 3 66%]** sites must report the check as failing in order to notify failure).

## Outage

An outage would include the site from which the event was reported. There could therefore be several outages for each check at the same time. The number of sites from which a check fails would be `COUNT(id) WHERE check_id = X AND failing_strikes >= failing_threshold`.

The failing condition at any given time would be `if (COUNT(id) WHERE check_id = X AND failing_strikes >= failing_threshold) >= failing_sites_threshold`.

We cannot rely on failing_strikes only to determine, after the fact, an actual outage, which is why we need another table to contain the history of confirmed outages, along with start and end time. This would split the `outage` table into two new tables:

site_outages
  id
  uuid
  check_id
  passing_strikes
  failing_strikes
  resolved

outages
  id
  uuid
  check_id
  started_on
  ended_on
  comment

The `site_outages` table would be used when receiving an event. The `outages` event would be used when an outage condition is triggered (and an alert sent).

## Runner

A runner would be identified by a controller-defined `slug` (e.g. `us-1`), this identifier would be included in a short-lived `JWT` signed with a private key generated on the controller. The controller, having the public key, is therefore capable of verifying a request comes from an authorized runner.

The runner would periodically ask the controller for `stale` checks should be run on that particular runner, through `GET /api/runner/checks`. The response should contain the specs of all checks that are due for running.

The runner would report the status for those checks to the controller, who would store (through `POST /api/runner/report`), evaluate, and optionnaly report them.

Two options:

 * A runner would wait until its local queue of stale checks is empty before requesting another batch. This, along with the network round-trip, could introduce delay in check handling (especially for short `interval`s). Such is life.
 * A runner would periodically poll for stale checks, storing `check_id` and the last event's `id` in order to prevent double-handling of the same check. This tuple `(check_id, event_id)` would be appended to the report in order for the controller to do the same.
