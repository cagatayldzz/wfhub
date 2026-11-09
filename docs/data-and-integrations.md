# Data and integrations

WFHub displays live data from external services. Availability, response shape, and image content are controlled by those services; network failures can affect individual screens.

## Sources

| Content | Source | Use |
| --- | --- | --- |
| World state and activities | `https://api.warframestat.us/{platform}/` | Home and activity data, including alerts and cycles |
| Codex catalogue and details | `https://wfhub.cagatayldzz.com/api/` | Category lists and item details |
| Recent market orders and items | `https://api.warframe.market/v2/` | Recent buy/sell orders joined with item names and icons |
| Artwork and banners | `https://wfhub.cagatayldzz.com/public/` and `/api/events.json` | Home banners, activity icons, and Codex artwork |
| Market artwork and item pages | `https://warframe.market/` | Item icons and links to market listings |

The supported platform codes in the iOS app are `pc`, `xb1`, `swi`, and `ps4`. These codes are used to construct world-state URLs.

## Client behavior

The iOS client uses Swift networking and `AsyncImage`. It stores preferences with `AppStorage`. Home, market, and activity screens periodically refresh while their tasks are active; data screens also provide pull-to-refresh.

The app should not assume that an optional API field or image is always present. When adding a screen, handle empty data and failed requests, and avoid displaying raw identifiers when a user-facing name is available.
