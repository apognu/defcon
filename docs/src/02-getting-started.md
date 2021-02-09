# Getting started

This guide will help you set up a simple Defcon instance, configure a first HTTP check and query the results through the bundled API.

## Requirements

The following list described the infrastructure required to follow this guide:

 * A Linux box
 * A MySQL instance, with a user account and an empty database

## Download a release

Binaries are listed under the [Releases](https://github.com/apognu/defcon/release) section of the GitHub repository. A new release will be created for each tag on the codebase, and a special `tip` release follow `master` and provides the latest snapshot of the code.

```shell
$ curl https://github.com/apognu/defcon/releases/download/tip/defcon-tip-x86_64 > defcon
$ chmod +x defcon
```

## Running the controller

From here, you can start the controller:

```shell
$ RUST_LOG=defcon=debug \
  DSN=mysql://defcon:password@mysql.host/defcon?ssl-mode=DISABLED \
  ./defcon
INFO[2021-02-06T11:48:51.801+0000] starting api process port="8000"
INFO[2021-02-06T11:48:51.801+0000] starting handler process interval="1s"
INFO[2021-02-06T11:48:51.801+0000] no public key found, disabling runner endpoints
```

## Creating your first check

In this guide, we will monitor two HTTP services, that will need to return a `200 OK` status code to pass. Each check will run every 10 seconds and will require three failures to be considered failed. We will use the Defcon API to create the checks.

Our first check can be represented with the following JSON:

```json
{
  "name": "Successful HTTP request",
  "interval": "10s",
  "sites": ["@controller"],
  "passing_threshold": 3,
  "failing_threshold": 3,
  "site_threshold": 1,
  "spec": {
    "kind": "http",
    "url": "http://jsonplaceholder.typicode.com/users",
    "code": 200
  }
}
```

This snippet defines the following:

 * The human-readable name for this check
 * The interval at which the check should be run (here, 10 seconds)
 * The sites on which the check will be run (`@controller` is the implicit name for Defcon's controller)
 * Each site will be considered as failed when the checks fails three times in a row, and recovered after three successes
 * An outage will be created when the number of failed sites reaches 1
 * This check uses the `http` handler, making a `GET` request to the provided URL, and expect a response status code of `200`

You can create the check by performing a `POST` request to `http://127.0.0.1:8000/api/checks`:

```shell
$ curl -v -XPOST http://127.0.0.1:8000/api/checks -d@book.json
HTTP/1.1 201 Created
location: /api/checks/82a3b532-0883-4544-ba2c-0a7159a89d8e
```

You can check the configuration for this check by calling the API with the returned path:

```shell
$ curl http://127.0.0.1:8000/api/checks/82a3b532-0883-4544-ba2c-0a7159a89d8e
{
    "alerter": null,
    "enabled": true,
    "failing_threshold": 3,
    "interval": "10s",
    "name": "Successful HTTP request",
    "passing_threshold": 3,
    "silent": false,
    "site_threshold": 0,
    "sites": [
        "@controller"
    ],
    "spec": {
        "code": 200,
        "content": null,
        "digest": null,
        "headers": {},
        "kind": "http",
        "timeout": null,
        "url": "http://jsonplaceholder.typicode.com/users"
    },
    "uuid": "82a3b532-0883-4544-ba2c-0a7159a89d8e"
}
```

## Check the check status

If you look at your console where Defcon is running, you should see that the handler for this check is running:

```shell
DEBG[2021-02-05T20:30:44.323+0000] check passed site="@controller" kind="http" check="82a3b532-0883-4544-ba2c-0a7159a89d8e" name="Successful HTTP request"
DEBG[2021-02-05T20:30:54.203+0000] check passed site="@controller" kind="http" check="f00ee7ad-b389-4819-bb24-e9797735e2df" name="Successful HTTP request"
DEBG[2021-02-05T20:30:54.207+0000] check passed site="@controller" kind="http" check="cf4a4917-92fb-4721-ab76-cae6a5fda2b8" name="Successful HTTP request"
```

## Create a failing check

As an exercice to the reader, create another check from the above model, but this time, define it as expecting a `201` status code. This check should fail and create an outage when `failing_threshold` is reached.

```shell
DEBG[2021-02-05T20:56:42.185+0000] check failed site="@controller" kind="http" check="4766d0dc-5d39-4ec7-8aee-95b46f33dc55" name="Personal - Website & API" message="status code was 200"
DEBG[2021-02-05T20:56:53.164+0000] check failed site="@controller" kind="http" check="4766d0dc-5d39-4ec7-8aee-95b46f33dc55" name="Personal - Website & API" message="status code was 200"
DEBG[2021-02-05T20:57:04.192+0000] check failed site="@controller" kind="http" check="4766d0dc-5d39-4ec7-8aee-95b46f33dc55" name="Personal - Website & API" message="status code was 200"
INFO[2021-02-05T20:57:04.291+0000] site outage started site="@controller" kind="http" check="4766d0dc-5d39-4ec7-8aee-95b46f33dc55" failed="3/3" passed="0/3"
INFO[2021-02-05T20:57:04.358+0000] outage confirmed check="4766d0dc-5d39-4ec7-8aee-95b46f33dc55" outage="f3bdec24-1f7f-4fd6-bbe8-2e937cc67746" since="2021-02-05 20:57:04 UTC"
```

Here you can see that a site outage was created, and since our `site_threshold` was set to `1`, an outage was confirmed for the check.
