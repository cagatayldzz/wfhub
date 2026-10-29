# Data and integrations

WFHub displays live data from external services. Availability, response shape, and image content are controlled by those services; network failures can affect individual screens.

## Sources

| Content | Source | Use |
| --- | --- | --- |
| World state and activities | `https://api.warframestat.us/{platform}/` | Home and activity data, including alerts and cycles |
| Codex catalogue and details | `https://wfhub.cagatayldzz.com/api/` | Category lists and item details |
| Recent market orders and items | `https://api.warframe.market/v2/` | Recent buy/sell orders joined with item names and icons |
| Artwork and banners | `https://cdn.cagatayldzz.com/constants/wfhub/` | Home banners, activity icons, and Codex artwork |
| Market artwork and item pages | `https://warframe.market/` | Item icons and links to market listings |

The supported platform codes in the shared core are `pc`, `xb1`, `swi`, and `ps4`. These codes are used to construct world-state URLs.

## Client behavior

The desktop client fetches data on a background executor and displays loading, error, and retry states. It caches downloaded images in a `wfhub-images` directory under the operating system's temporary directory. Cached files may be removed by the operating system and downloaded again. Desktop settings are stored in `~/.config/wfhub/settings.json`; they are separate from iOS settings.

The iOS client uses Swift networking and `AsyncImage`. It stores preferences with `AppStorage`. Home, market, and activity screens periodically refresh while their tasks are active; data screens also provide pull-to-refresh. The two clients do not share cached data or preferences.

Neither client should assume that an optional API field or image is always present. When adding a screen, handle empty data and failed requests, and avoid displaying raw identifiers when a user-facing name is available.
