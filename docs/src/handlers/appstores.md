# Application stores

Two handlers exist to verify the availability of Android and iOS application on, respectively, the Play Store and the App Store. These can be used to monitor for Google or Apple removing your apps, as well as human error or malice.

# App Store

## Attributes

| Attribute   | Type   | Example            | Description                          |
| ----------- | ------ | ------------------ | ------------------------------------ |
| `kind`      | string | `"app_store"`      | -                                    |
| `bundle_id` | string | `"com.apple.Maps"` | Bundle ID for the iOS app to monitor |

# Play Store

## Attributes

| Attribute | Type   | Example                          | Description                                   |
| --------- | ------ | -------------------------------- | --------------------------------------------- |
| `kind`    | string | `"play_store"`                   | -                                             |
| `app_id`  | string | `"com.google.android.apps.maps"` | Application ID for the Android app to monitor |
