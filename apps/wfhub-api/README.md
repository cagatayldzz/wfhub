# WFHub API

The Rust API behind `https://wfhub.cagatayldzz.com/api`. It serves the WFHub Codex catalogue, mastery snapshots, event
banners and a feedback endpoint. For local development, run `npm run dev:web` and open `http://localhost:8000/api/` for the
Swagger documentation.

**Everything is static.** The data are files in `apps/wfhub-api/data`, which `scripts/build-site.sh` copies into the site Cloudflare serves (at `/data`). The Worker is compiled **once** (its
compiled form, `worker/dist`, is committed), holds no data and only serves those files; the only thing it writes is
feedback (Cloudflare D1). To change the API you edit a file and deploy the site, and nothing is compiled again.

```text
src/routes.rs           the routes, as plain functions            src/feedback.rs   what feedback is accepted
src/catalogue.rs        makes the catalogue files (tool and tests only)
catalogue/fetch.sh      downloads the WFCD sources into catalogue/sources (not committed)
migrations/             D1 migrations (feedback)
worker/                 the Cloudflare Worker (Rust, WASM): glue between the site's files and the routes
worker/dist/            the compiled Worker (index.js, index_bg.wasm), committed
data/                   what the Worker serves
  events.json           the banners (edit by hand)
  mastery/<name>.json   mastery checklist snapshots (edit by hand)
  catalogue/            made by catalogue:export from WFCD: <category>.json and <category>/<first character of slug>.json
apps/wfhub-web/swagger  the Swagger page and openapi.json (edit by hand)
```

```sh
npm run dev:web           # the real thing locally (wrangler dev): the website and /api together, on port 8000
npm run api:test          # routes, feedback rules and the static files
npm run api:build         # compiles the Worker into worker/dist: only when worker/ or src/routes.rs, src/feedback.rs change
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
| `POST /api/feedback` | stores anonymous feedback |

Anything else answers `404`, and a wrong method `405`. Every answer carries `Access-Control-Allow-Origin: *`.

## Codex catalogue

`npm run catalogue:fetch` downloads the category files of [WFCD / warframe-items](https://github.com/WFCD/warframe-items) and
`npm run catalogue:export` turns each into a list (`name`, `imageName`, `slug`) and bucket files with the items, named by
the first character of the slug, so the Worker never reads more than a small file. Duplicate names are dropped. Commit the
result and deploy; the catalogue changes only when you run it again.

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
(`migrations/`) before deploying; a failed migration stops the deploy. Set a long random `FEEDBACK_RATE_LIMIT_SECRET` with
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
