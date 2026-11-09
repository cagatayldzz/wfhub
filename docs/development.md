# Development

## Prerequisites

Install Bun and Node.js for web development. iOS development requires Xcode, XcodeGen, and an iOS 17 or later simulator or device.

## Run the clients

To generate the iOS project and open it in Xcode:

```sh
cd apps/wfhub-mobile
xcodegen generate
open WFHub.xcodeproj
```

Select the `WFHub` scheme and a simulator or device. Physical devices require a valid signing team.

Alternatively, from the repository root, `bash scripts/mobile.sh simulator` generates the project, builds the app, and launches the default iPhone 15 simulator. Set `WFHUB_SIMULATOR` to another device name or UDID when needed. The simulator script requires a matching available simulator.

## Website and API

Run `bun install`, then `bun run dev:web` from the repository root and open `http://localhost:8000`. The site uses TanStack React Start, Vite and Tailwind CSS; its Vite development server also handles API requests. API requests use locally generated data when available, otherwise they use the deployed API. Run `bun run cf:build` to build the site and generate local data; `bun run cf:deploy` builds and deploys the website and API to Cloudflare. The build produces the client in `dist/client` and the Worker in `dist/server`. Tailwind's CSS input is generated under `.tanstack` when Vite starts.

The privacy and terms pages are drafts and require review before publication. See the [API README](../apps/wfhub-web/api/README.md) for configuration and dataset details.

## Checks

```sh
bunx tsc --noEmit
bun test apps/wfhub-web/api
```

`bash scripts/test.sh` runs these checks in sequence. API tests cover routes, HTTP methods, and response behavior; they do not replace UI and device testing. When changing UI behavior, exercise the affected screens and navigation in the relevant client.

## Generated files and release

The iOS `build/` directory and generated `.xcodeproj` files are ignored by Git. Edit `apps/wfhub-mobile/project.yml` and regenerate the project instead of editing the generated project as the source of truth.

`bash scripts/mobile.sh archive` generates an iOS archive and increments `CURRENT_PROJECT_VERSION` in `project.yml` by default. Invoke the script directly with `--no-bump` to retain the current number. Archiving requires an appropriately configured signing team.

If the simulator script cannot find iPhone 15, select an available simulator with `WFHUB_SIMULATOR`. Network-backed screens require access to the services listed in [Data and integrations](data-and-integrations.md).

The TypeScript development and Cloudflare scripts use Bun.
