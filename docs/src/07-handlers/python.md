# Python

This handler executes an external Python script to perform the actual check.

This script must contain a `check()` funtion that returns the status and message of the check. The constants `OK`, `WARNING` and `CRITICAL` are provided in the current module.

The handler looks for a file named `<script>.py`, so the script name must be provided without the extension.

```python
def check():
  return (CRITICAL, "something unexpected happened")
```

## Attributes

| Attribute | Type   | Example            | Description                                          |
| --------- | ------ | ------------------ | ---------------------------------------------------- |
| `script`  | string | `"mycustomscript"` | The extension-stripped name of the script to execute |

## Configuration

The path where the script are looked up in can be configured through the `SCRIPTS_PATH` environment variable, which defaults to `/var/lib/defcon/scripts`.
