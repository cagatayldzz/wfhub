# WFHub API

The API behind `https://wfhub.cagatayldzz.com/api`: the Warframe Codex catalogue, mastery checklists, event banners, the
**live game state** (fissures, cycles, Baro, ...) and a feedback endpoint. Swagger documentation: `/api/`.

- [How it works](#how-it-works)
- [Project layout](#project-layout)
- [Commands](#commands)
- [Routes](#routes)
- [World state](#world-state)
- [Codex catalogue](#codex-catalogue)
- [Mastery and events](#mastery-and-events)
- [Feedback](#feedback)
- [Deploying](#deploying)

## How it works

The site and the API are **one Cloudflare Worker** (`wrangler.jsonc` at the repository root). `/api` and `/api/*` go to the Worker, which is
Rust compiled to WASM; everything else is served from the static site (`apps/wfhub-web`).

- **Almost everything is a static file.** The catalogue, mastery lists, banners and name tables are files in `apps/wfhub-api/data`.
  `scripts/build-site.sh` copies them into the site (served at `/data`) and the Worker only reads them. To change the data you edit a file and deploy.
- **The live game state is the exception.** The Worker fetches it from Warframe's server (through a proxy), keeps it in D1 and parses it per request
  (see [World state](#world-state)).
- **Feedback** is the only thing users write (D1).
- **The compiled Worker is committed** (`worker/dist`): Cloudflare's build has no Rust. `cf:deploy` refuses to deploy a `dist` that does not match
  the Rust sources (`scripts/check-worker.sh`), so run `npm run api:build` and commit the result whenever `src/`, `worker/src/` (Rust or `tunnel.js`)
  or a `Cargo` file changes.

## Project layout

```text
src/                      the library: everything that can be tested without the Worker
  lib.rs
  routes.rs               the routes as plain functions (method + path -> what to answer)
  feedback.rs             which feedback is accepted
  catalogue.rs            makes the catalogue files (tool and tests only)
  worldstate/             the live game state parser (pure functions of the raw state and the names)
    mod.rs                platforms, sections, parse_section / parse_all
    names.rs              names of nodes, missions, items (tables + shards)
    value.rs              reading the raw state: dates, ids, rewards, the fields every timed thing has
    cycles.rs             Cetus, Cambion, Earth, Vallis, Zariman, Duviri
    missions.rs           fissures, sortie, archon hunt, alerts, invasions, arbitration, sentient outposts
    store.rs              Baro Ki'Teer, daily deals, flash sales, Steel Path
    community.rs          events, news, Nightwave, Simaris, construction
  bin/                    export-catalogue, export-worldstate: make the files in data/
worker/                   the Cloudflare Worker: glue between the site's files, D1, the game and the library
  src/lib.rs              the router
  src/worldstate.rs       fetch, store and cache the game state
  src/tunnel.js           HTTPS through an HTTP proxy (CONNECT tunnel), see World state
  src/feedback.rs         feedback in D1
  src/access.rs           Cloudflare Access check for the admin route
  migrations/             D1 migrations (feedback, worldstate)
  dist/                   the compiled Worker, committed
data/                     what the Worker serves (copied to /data)
  events.json             the banners (by hand)
  mastery/<name>.json     mastery checklists (by hand)
  catalogue/              made by catalogue:export: <category>.json and <category>/<first character of slug>.json
  worldstate/             made by worldstate:export: tables.json and names/<shard>.json
tests/                    routes, feedback, static data, the parser (fixtures/ has a real state of every platform)
.sources/                 downloaded WFCD sources, not committed (scripts/fetch-*.sh at the repository root)
```

The Swagger page and `openapi.json` are in `apps/wfhub-web/swagger` (edited by hand).

## Commands

Run from the repository root.

```sh
npm run dev:web             # the website and /api together on http://localhost:8000 (wrangler dev)
npm run api:test            # cargo test: routes, feedback, static files, the parser
npm run api:build           # compile the Worker into worker/dist (commit the result)
npm run format:rust
npm run catalogue:fetch     # download the WFCD item files
npm run catalogue:export    # write data/catalogue
npm run worldstate:fetch    # download warframe-worldstate-data (nodes, languages)
npm run worldstate:export   # write data/worldstate
npm run cf:deploy           # check dist, build the site, apply D1 migrations, deploy
```

## Routes

All under `/api`. Every answer carries `Access-Control-Allow-Origin: *`; anything else is `404`, a wrong method `405`.

| Route | Answer |
| --- | --- |
| `GET /api/` | Swagger UI (`/api/swagger/openapi.json` is the description) |
| `GET /api/health` | `"ok"` |
| `GET /api/{category}.json` | the items of a Codex category, e.g. `/api/warframes.json` |
| `GET /api/{category}/{slug}.json` | one item, e.g. `/api/warframes/excalibur.json` |
| `GET /api/events.json` | the banners (`data/events.json`) |
| `GET /api/mastery/{category}.json` | a mastery checklist; `/api/mastery/updated.json` says when each was made |
| `GET /api/worldstate/{platform}.json` | every section of the live game state. Platforms: `pc`, `xb1`, `ps4`, `swi` (`ps5` and `mob` are aliases) |
| `GET /api/worldstate/{platform}/{section}.json` | one section: alerts, arbitration, archonHunt, cambionCycle, cetusCycle, constructionProgress, dailyDeals, duviriCycle, earthCycle, events, fissures, flashSales, invasions, news, nightwave, sentientOutposts, simaris, sortie, steelPath, vallisCycle, voidTrader, zarimanCycle |
| `POST /api/feedback` | stores anonymous feedback |
| `GET /api/admin/feedback` | lists feedback (behind Cloudflare Access, not public) |

## World state

`/api/worldstate/...` replaces `api.warframestat.us`. The Worker parses Warframe's own `worldState.php`
(`api.warframe.com`, `api-xb1`, `api-ps4`, `api-swi`; all `https://{host}.warframe.com/cdn/worldState.php`, about 165 KB, 27 KB gzipped) into the
same JSON shapes as warframe-status. The parser is pure Rust (`src/worldstate/`, tested with real states in `tests/fixtures`).

### The path of a request

```text
request ─▶ 1. parsed answer in the Cache API (60 s)         hit: answered, nothing is parsed
              │ miss
              ▼
           2. raw state in the Cache API (5 min)            hit: parse it
              │ miss
              ▼
           3. raw state in D1, table `worldstate`           younger than 5 min: parse it
              │ older or missing                            (one copy shared by all data centres)
              ▼
           4. the game, through the proxy                   store it in D1 and in the cache, parse it
              │ proxy or game fails
              ▼
           5. the last copy in D1, flagged `x-worldstate-stale`; `503` with a `reason` only when there is none
```

Headers: `x-worldstate-age` is the age of the copy (seconds since it was fetched from the game); `x-worldstate-stale: 1` means it is over 30 minutes old.

**Nothing in an answer depends on the clock**: no `eta`, `timeLeft`, `expired`, `active`, `timestamp` or countdown strings. The client counts down from
`activation` and `expiry`. That is why a parsed answer can be cached and the same for every request until the game's state changes.

### The proxy

The game's server answers an empty `403` to Cloudflare's addresses (and to GitHub Actions, Azure, AWS), whatever the headers. So the Worker reaches
it through an HTTP proxy, [Webshare](https://dashboard.webshare.io/dashboard) (free plan: 10 shared proxies, 1 GB of traffic a month).

`fetch` cannot use a proxy, and workers-rs's `start_tls()` cannot say which host the certificate must match. So `worker/src/tunnel.js` does it with the
sockets API: connect to the proxy, `CONNECT {host}:443` with `Proxy-Authorization`, `startTls({ expectedServerHostname })`, `GET`, then decode chunked and gzip.
(Plain `http://` through the proxy does not work, the game redirects to https.)

Setup, once:

1. Sign in at <https://dashboard.webshare.io/dashboard>. **Proxy > Settings** shows the *Proxy Username* and *Proxy Password*. If they ever leak, change them
   there and update the secret.
2. The secret is the **rotating** address, so one value works whichever of the proxies the game accepts:
   `http://{username}-rotate:{password}@p.webshare.io:80/` (the username with `-rotate`, host `p.webshare.io`, port `80`).
3. Set it on the Worker. You type the value, it never goes into the repository:
   `npx wrangler secret put WORLDSTATE_PROXY`, or Cloudflare dashboard > Workers > wfhub > Settings > Variables and Secrets.
4. Locally, put `WORLDSTATE_PROXY=...` in `.dev.vars` (gitignored), run `npx wrangler d1 migrations apply DB --local`, then `npm run dev:web`.

Test a proxy by hand (should print `200`):

```sh
curl --proxy 'http://USERNAME-rotate:PASSWORD@p.webshare.io:80' -o /dev/null -w '%{http_code}\n' https://api.warframe.com/cdn/worldState.php
```

Traffic: a fetch is about 27 KB. A platform is fetched at most once in 5 minutes (the D1 copy is shared by all data centres), so four platforms requested around
the clock cost at most about 0.9 GB a month. The traffic left is in the Webshare dashboard. When it runs out the Worker keeps answering from the last copy.

### Names and constants

Names (nodes, mission types, items, challenges) come from `data/worldstate`: `tables.json` and 32 name shards (`names/{n}.json`, by hash of the path); a request loads only
the shards its state needs. Refresh them when the game adds content:

```sh
npm run catalogue:fetch      # warframe-items (item names)
npm run worldstate:fetch     # warframe-worldstate-data (nodes, languages)
npm run worldstate:export    # writes data/worldstate
```

Not in the raw state: `arbitration` answers the "unknown" placeholder (as warframe-status does without its extra source); the Vallis, Earth and Duviri constants are the
ones of the WFCD parser; Nightwave reputation is 1000 (daily), 4500 (weekly) and 7000 (elite). The raw state only has English names, so `news.translations` has `en` only.

## Codex catalogue

`npm run catalogue:fetch` downloads the category files of [WFCD / warframe-items](https://github.com/WFCD/warframe-items) and `npm run catalogue:export` turns each into
a list (`name`, `imageName`, `slug`) plus bucket files with the items, named by the first character of the slug, so the Worker never reads more than a small file.
Duplicate names are dropped. Commit the result and deploy; the catalogue changes only when you run it again.

## Mastery and events

`GET /api/mastery/warframes.json` returns the main Warframe checklist as an array of `{ "name", "image", "level", "mastery" }`. It is a manually maintained snapshot of
the [Warframe Wiki checklist](https://wiki.warframe.com/w/Mastery_Rank_Checklist), checked against the main Warframe table on 2026-11-07 (wiki page last edited
2026-10-23); Founder exclusives are in a separate wiki section and are not included. Image URLs point to the thumbnails shown in the wiki checklist. Update
`data/mastery/warframes.json` and deploy when the checklist changes. Wiki content is available under
[CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/); additional terms apply.

The same format is available for `primary`, `secondary`, `melee`, `kitgun`, `zaw`, `amp`, `sentinel`, `moa`, `hound`, `beast`, `robotic`, `archwing`, `archgun`,
`archmelee`, `necramech`, `railjack` and `k-drive`. `image` is null for entries without an image in the wiki checklist (Plexus and the five K-Drives). Further endpoints:
`railjack-intrinsic`, `drifter-intrinsic`, `starchart`, `steel-path`, `founders` and `china-founders`; they have a stable `id` and an optional `group`, and `level` is null
for missions and founder items. Update the dates in `updated.json` when the data is refreshed.

`/api/events.json` serves the banners of `data/events.json`. Edit that file and deploy to change them.

## Feedback

`POST /api/feedback` with `Content-Type: application/json` and the fields `title` (3-120 characters), `category` (`bug`, `suggestion` or `other`) and `message`
(10-4000 characters); the body is at most 8 KiB. It answers `201`. Feedback goes to the `DB` D1 binding and is inspected in the D1 dashboard or at `/api/admin/feedback`.

The rate limit is five submissions per client per hour, using daily-rotated keyed hashes of the IP address instead of stored addresses; it is a basic abuse mitigation,
not authentication. Set a long random secret once, the endpoint answers `503` without it:

```sh
npx wrangler secret put FEEDBACK_RATE_LIMIT_SECRET
```

Locally, put it in `.dev.vars` and run `npx wrangler d1 migrations apply DB --local`.

## Deploying

`npm run cf:deploy` (or a push to `main` with the build command `npm run cf:build` and the deploy command `npm run cf:deploy`) checks that `worker/dist` is fresh, builds the
site, applies the pending remote D1 migrations (a failed migration stops the deploy) and uploads everything. It never compiles Rust.

Secrets of the Worker: `WORLDSTATE_PROXY` (game state) and `FEEDBACK_RATE_LIMIT_SECRET` (feedback).

Warframe and related names and assets belong to their respective rights holders, including Digital Extremes. WFHub is an independent community project.
