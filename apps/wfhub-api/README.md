# WFHub API

The Rust API behind `https://wfhub.cagatayldzz.com/api`. It serves the WFHub Codex catalogue, mastery snapshots, event
banners, the live game state (fissures, cycles, Baro...) and a feedback endpoint. For local development, run `npm run dev:web` and open `http://localhost:8000/api/` for the
Swagger documentation.

**Almost everything is static, except the live game state.** The data are files in `apps/wfhub-api/data`, which `scripts/build-site.sh` copies into the site Cloudflare serves (at `/data`). The Worker is Rust compiled to WASM (`npm run api:build`; the result, `worker/dist`, is committed and checked before every deploy), holds no data and only serves those files; it writes feedback and the pushed game state (Cloudflare D1) and parses that state live (see World state below). To change the data you edit a file and deploy.

```text
src/
  lib.rs                the library: everything testable without the Worker
  routes.rs             the routes, as plain functions (path -> what to answer)
  feedback.rs           what feedback is accepted
  catalogue.rs          makes the catalogue files (tool and tests only)
  worldstate/           the live game state parser
    mod.rs              platforms, endpoints, parse_section / parse_all
    names.rs            names of nodes, missions, items (tables + shards)
    value.rs            reading the raw state: dates, ids, rewards, the fields of every timed thing
    cycles.rs           Cetus, Cambion, Earth, Vallis, Zariman, Duviri
    missions.rs         fissures, sortie, archon hunt, alerts, invasions, arbitration, sentient outposts
    store.rs            Baro Ki'Teer, daily deals, flash sales, Steel Path
    community.rs        events, news, Nightwave, Simaris, construction
  bin/                  export-catalogue, export-worldstate (make the files in data/)
worker/                 the Cloudflare Worker (Rust, WASM): glue between the site's files, the game and the library
  migrations/           D1 migrations (feedback)
  src/lib.rs            the router            src/feedback.rs   feedback in D1
  src/worldstate.rs     game state in D1      src/access.rs     Cloudflare Access check
  dist/                 the compiled Worker, committed (rebuild with npm run api:build when Rust changes)
data/                   what the Worker serves (copied to /data by scripts/build-site.sh)
  events.json           the banners (edit by hand)
  mastery/<name>.json   mastery checklist snapshots (edit by hand)
  catalogue/            made by catalogue:export from WFCD: <category>.json and <category>/<first character of slug>.json
  worldstate/           made by worldstate:export: tables.json and names/<shard>.json
tests/                  routes, feedback, static data and the worldstate parser (fixtures/ has real states of every platform)
.sources/               the downloaded WFCD sources (not committed); scripts/fetch-*.sh at the repository root download them
apps/wfhub-web/swagger  the Swagger page and openapi.json (edit by hand)
```

```sh
npm run dev:web           # the real thing locally (wrangler dev): the website and /api together, on port 8000
npm run api:test          # routes, feedback rules and the static files
npm run api:build         # compiles the Worker into worker/dist: when src/ or worker/src change; commit the result (cf:deploy checks it is fresh)
npm run catalogue:fetch   # downloads the WFCD files
npm run catalogue:export  # writes data/catalogue
npm run format:rust
```

## Routes (under `/api`)

| Route | Answer |
| --- | --- |
| `GET /api/` | Swagger UI (`/api/swagger/openapi.json` is the description) |
| `GET /api/health` | `"ok"` |
| `GET /api/{category}.json` | the items of a Codex category, e.g. `/api/warframes.json` |
| `GET /api/{category}/{slug}.json` | one item, e.g. `/api/warframes/excalibur.json` |
| `GET /api/events.json` | the banners, maintained in `data/events.json` |
| `GET /api/mastery/{category}.json` | a mastery snapshot; `/api/mastery/updated.json` says when it was made |
| `PUT /api/ingest/worldstate/{platform}` | stores the raw game state (bearer token; used by `scripts/push-worldstate.sh`) |
| `GET /api/worldstate/{platform}.json` | every section of the live game state (`pc`, `xb1`, `ps4`, `swi`; `ps5` and `mob` are aliases) |
| `GET /api/worldstate/{platform}/{section}.json` | one section: alerts, arbitration, archonHunt, cambionCycle, cetusCycle, constructionProgress, dailyDeals, duviriCycle, earthCycle, events, fissures, flashSales, invasions, news, nightwave, sentientOutposts, simaris, sortie, steelPath, vallisCycle, voidTrader, zarimanCycle |
| `POST /api/feedback` | stores anonymous feedback |

Anything else answers `404`, and a wrong method `405`. Every answer carries `Access-Control-Allow-Origin: *`.

## Codex catalogue

`npm run catalogue:fetch` downloads the category files of [WFCD / warframe-items](https://github.com/WFCD/warframe-items) and
`npm run catalogue:export` turns each into a list (`name`, `imageName`, `slug`) and bucket files with the items, named by
the first character of the slug, so the Worker never reads more than a small file. Duplicate names are dropped. Commit the
result and deploy; the catalogue changes only when you run it again.

## World state

`/api/worldstate/...` replaces `api.warframestat.us`. It parses Warframe's own `worldState.php` (pc `api.warframe.com`, xb1
`api-xb1.warframe.com`, ps4 `api-ps4.warframe.com`, swi `api-swi.warframe.com`, all under `/cdn/worldState.php`) into the same JSON
shapes as warframe-status (`src/worldstate/`; pure Rust, `cargo test`, fixtures in `tests/fixtures`).

**The game's server answers 403 to Cloudflare's addresses** (and to GitHub Actions/Azure; an empty 403 whatever the headers), and a
Worker cannot go through a proxy (its TLS cannot name the host the certificate must match), so the Worker does not fetch the state. Instead
`.github/workflows/worldstate.yml` runs `scripts/push-worldstate.sh` on GitHub every 10 minutes: it downloads the four raw states with
`curl` **through a proxy** (`WORLDSTATE_PROXY`, a Webshare free-plan proxy: 10 shared addresses, 1 GB a month; one run is about
4 x 27 KB gzipped, ~0.5 GB a month) and `PUT`s each to `/api/ingest/worldstate/{platform}` with a bearer token. The Worker keeps it in D1
(table `worldstate`). The pushes go to the Worker's **workers.dev address** (`wrangler.jsonc` has `workers_dev: true`; the repository
variable `WFHUB_INGEST_URL` holds it): the Bot Fight Mode of the `cagatayldzz.com` zone (free plan, it cannot be skipped by a rule) gives
GitHub's addresses a managed challenge, which a `PUT` cannot solve. The token still protects the route.

Reading: the raw state is read from D1 and **cached 5 minutes per platform in each data centre** (Cache API), then parsed per request.
Answers carry `x-worldstate-age` (seconds since the push) and `x-worldstate-stale: 1` over 30 minutes; times in the answers come from the
clock, so a copy a few minutes old is still right about what has expired. Before the first push the endpoints answer `503` with a `reason`.
GitHub may delay scheduled runs by several minutes.

Setup, once. The token is a long random value (`openssl rand -hex 32`) shared by the Worker and the workflow; the push route answers
`401` without it and `503` while the Worker has none:

```sh
npx wrangler secret put WORLDSTATE_PUSH_TOKEN     # the Worker (or Cloudflare dashboard > Workers > wfhub > Settings > Variables and Secrets)
gh secret set WORLDSTATE_PUSH_TOKEN               # the repository, same value (or GitHub > Settings > Secrets and variables > Actions)
gh secret set WORLDSTATE_PROXY                    # http://user:password@host:port  (repository only; the Worker does not use it)
```

By hand: `WORLDSTATE_PUSH_TOKEN=... WORLDSTATE_PROXY=... bash scripts/push-worldstate.sh` (`WFHUB_URL=http://localhost:8000` pushes to a
local `npm run dev:web`, with the token in `.dev.vars`).

Names (nodes, mission types, items, challenges) come from `data/worldstate`: `tables.json` and 32 name shards
(`names/{n}.json`, by hash of the path). A request loads only the shards its state needs. Refresh them when the game adds content:

```sh
npm run catalogue:fetch      # warframe-items (item names)
npm run worldstate:fetch     # warframe-worldstate-data (nodes, languages)
npm run worldstate:export    # writes data/worldstate
```

Not in the raw state: `arbitration` answers the "unknown" placeholder (as warframe-status does without its extra source); the
Vallis, Earth, Duviri constants are the ones of the WFCD parser; Nightwave reputation is 1000 (daily), 4500 (weekly) and 7000 (elite).
The raw state only has English names here; `news.translations` has `en` only.

## Mastery and events

`GET /api/mastery/warframes.json` returns the main Warframe checklist as an array of `{ "name", "image", "level", "mastery" }`. It is a
manually maintained snapshot of the [Warframe Wiki checklist](https://wiki.warframe.com/w/Mastery_Rank_Checklist), checked against the
main Warframe table on 2026-11-07 (wiki page last edited 2026-10-23); Founder exclusives are in a separate wiki section and are not
included. Image URLs point to the thumbnails shown in the wiki checklist. Update `data/mastery/warframes.json` and deploy when the
checklist changes. Wiki content is available under [CC BY-NC-SA 3.0](https://creativecommons.org/licenses/by-nc-sa/3.0/); additional
terms apply.

The same snapshot format is available for `primary`, `secondary`, `melee`, `kitgun`, `zaw`, `amp`, `sentinel`, `moa`, `hound`, `beast`,
`robotic`, `archwing`, `archgun`, `archmelee`, `necramech`, `railjack`, and `k-drive`. The `image` field is null for entries without an
image in the wiki checklist (Plexus and the five K-Drives). Additional endpoints: `railjack-intrinsic`, `drifter-intrinsic`, `starchart`,
`steel-path`, `founders`, and `china-founders`. These include stable `id` and optional `group`; `level` is null for missions and founder
items. Snapshot dates in `updated.json` must be updated when the data is refreshed.

`/api/events.json` serves the banners maintained in `data/events.json`. Update that file and deploy to change them.

## Feedback

The Worker writes anonymous feedback to the `DB` D1 binding. `npm run cf:deploy` applies pending remote D1 migrations
(`worker/migrations/`) before deploying; a failed migration stops the deploy. Set a long random `FEEDBACK_RATE_LIMIT_SECRET` with
`npx wrangler secret put FEEDBACK_RATE_LIMIT_SECRET`; the endpoint answers 503 without it. Locally, put it in `.dev.vars` and run
`npx wrangler d1 migrations apply DB --local`.

Feedback is write-only via `POST /api/feedback` with `Content-Type: application/json` and the fields `title` (3-120 characters),
`category` (`bug`, `suggestion` or `other`) and `message` (10-4000 characters); the body is at most 8 KiB. It answers `201`. Inspect
submissions in the D1 dashboard. The rate limit is five submissions per client per hour, using daily-rotated keyed hashes of the
IP address instead of stored addresses; it is a basic abuse mitigation, not authentication.

## Deploying

The site and the API are one Worker (`wrangler.jsonc` at the root): `/api` and `/api/*` go to the compiled Worker, everything else
is served from `apps/wfhub-web`. Deploying only uploads files (`npm run cf:deploy`, or a push to `main` with the build command
`npm run cf:build` and the deploy command `npm run cf:deploy`); it never compiles Rust.

Warframe and related names and assets belong to their respective rights holders, including Digital Extremes. WFHub is an
independent community project.
