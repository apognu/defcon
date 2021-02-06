# Off-site runner

By default, all checks are run on the controller node, and require only one site outage for an outage to be confirmed. Defcon allows to offload check handling to other nodes by the use of off-site runners.

A runner is a stripped-down instance of Defcon that only knows how to run handlers and report their status back to the controller. Its workflow is the following:

 * Regularly check with the controller for stale checks that are configured to run on this particular runner
 * Call the handlers for each of those checks
 * Report their status back to the controller
 * Start over

An off-site runner authenticates itself to the controller by possessing the private key matching the controller's public key.

Whereas the controller is identified by the static `@controller` tag, each runner must be configured to have a unique tag, such as `eu-1` or `home-runner`. Site identifiers should only contain lowercase alphanumeric characters and dashes (`^[a-z0-9-]+$`)

## Download the binary

```shell
$ curl https://github.com/apognu/defcon/releases/download/tip/defcon-tip-x86_64 > defcon
$ chmod +x defcon
```

## Start a runner

```shell
$ PRIVATE_KEY=./defcon-private.pem \
  CONTROLLER_URL=http://127.0.0.1:8000 \
  SITE=eu-1 \
  ./defcon-runner
INFO[2021-02-06T14:18:36.973+0000] starting runner process site="eu-1" poll_interval="1s"
```

This runner will start running any stale check configured to run on site `eu-1`.
