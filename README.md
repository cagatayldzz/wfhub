# <img src="apps/wfhub-web/assets/icon.png" alt="" width="48" align="absmiddle"> WFHub

A Warframe companion for iOS, built with SwiftUI and a Rust API.

WFHub brings live activities, world states, the Codex catalogue, and recent market
orders together in one place. Follow events and cycles, explore items, and check buy
and sell orders from your iPhone.

## Get started

Download WFHub from the [App Store](https://apps.apple.com/app/wfhub/id6738619973?platform=iphone),
or follow the [development guide](docs/development.md) to run it locally. iOS
development requires Xcode, XcodeGen, and an iOS 17 or later simulator or device.

The website and API can also be run locally with Node.js and Rust:

```sh
npm run dev:web
```

Open [localhost:8000](http://localhost:8000) to view the site and API.

## Documentation

- [Development](docs/development.md) — prerequisites, setup, checks, and troubleshooting
- [Architecture](docs/architecture.md) — website, API, and iOS app
- [Data and integrations](docs/data-and-integrations.md) — APIs, images, storage, and refresh behavior
- [API](apps/wfhub-api/README.md) — routes, data, feedback, and deployment
- [Asset notices](docs/assets.md) — third-party artwork and usage considerations
- [Changelog](apps/wfhub-mobile/changelog) — iOS release notes

## License

WFHub is licensed under the [MIT License](LICENSE). Third-party assets and trademarks
have separate rights; see the [asset notices](docs/assets.md).
