# HTTP request

The HTTP handler will perform an HTTP GET request, with the specified parameter, and check a variety of elements from the response, namely, the status code, the string content and a digest of the response.

## Attributes

| Attribute | Type                | Example                     | Description                                    |
| --------- | ------------------- | --------------------------- | ---------------------------------------------- |
| `kind`    | string              | `"http"`                    | -                                              |
| `url`     | string              | `"https://example.com"`     | Full URL to request                            |
| `headers` | map<string, string> | `{ "authorization": "me" }` | List of headers to add to the request          |
| `timeout` | int                 | `2`                         | Abort the request after this number of seconds |
| `code`    | int                 | `201`                       | Status code of the response                    |
| `content` | string              | `"ACME"`                    | Substring to find in the response body         |
| `digest`  | string              | `"..."`                     | Hex-encoded SHA-512 sum of the response body   |
