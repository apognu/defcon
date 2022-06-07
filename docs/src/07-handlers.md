# Handlers

A handler is a process that knows how to determine the status of a particular kind of external service. A handler can, for example, perform an HTTP request, open a TCP connection, or check the presence of some domain-specific item on a remote server. The exact list of supported handler is described in the next sections.

A handler specification is a series of attributes that describes how to perform the check, and conditions on when to report an issue (for example, `when the status code is not 200`).

Valid attributs varies from handler to handler, but they all have one common attribute, `kind`, which specifies the kind of handler this is.

In the context of a check, a handler specification is laid out as:

```json
{
  // Check attributes
  "spec": {
    "kind": "<handler type>",
    // Spec attributes
  }
}
```
