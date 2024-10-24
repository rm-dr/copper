<p align="center">
  <a href="https://github.com/rm-dr/copper"><img src="./copperc/public/banner.svg" alt="Logo" width="60%"></a>
</p>

<div align="center">

![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/rm-dr/copper)
![GitHub Repo stars](https://img.shields.io/github/stars/rm-dr/copper)
![GitHub License](https://img.shields.io/github/license/rm-dr/copper)

**Copper** is the universal, automatic digital library.

</div>

---

## Features

- Polished web ui
  - Simple drag-and-drop uploading
  - Graphical pipeline editor
- Replaces
  - [Paperless](https://github.com/paperless-ngx/paperless-ngx) (documents)
  - [Calibre](https://github.com/kovidgoyal/calibre) (ebooks)
  - [Beets](https://github.com/beetbox/beets) (music)
  - [Photoview](https://github.com/photoview/photoview) (photos)
- <span style="color:grey">~~Scriptable api~~ (planned)</span>
- <span style="color:grey">~~Simple export & backup~~ (planned)</span>
- <span style="color:grey">~~Email notifications~~ (planned)</span>
- <span style="color:grey">~~Automatic ingestion~~ (planned)</span>
- <span style="color:grey">~~Fast & powerful search~~ (planned)</span>
- <span style="color:grey">~~Expose data for other services~~ (planned)</span>

## Non-features

Functionality that is intentionally omitted from Copper. Use a better tool.

- Low-code workflows
  - Copper processes data, not logic.
  - **Instead, use** [Node-red](https://github.com/node-red/node-red) or [n8n](https://github.com/n8n-io/n8n).
- Advanced playback UI
  - Copper's ui is designed to _manage_ media, not _consume_ it.
  - **Instead, use** [Jellyfin](https://jellyfin.org/).

## Getting started

Copper is designed to be run in docker. To get started, to the following:

- Download the contents of [`./dist`](./dist/)
- `cd dist`
- `docker compose up -d`
- Read `./dist/docker-compose.yml`
  If you want to run the stack manually, see [`CONTRIBUTING.md`](./CONTRIBUTING.md).

## User Documentation

Has yet to be written. It will likely be embedded in copper's web ui.

## Bugs and Feature requests

Please [open an issue](https://github.com/rm-dr/copper/issues).
