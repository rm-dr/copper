<p align="center">
  <a href="https://github.com/rm-dr/copper"><img src="./copperc/public/banner.svg" alt="Logo" width="60%"></a>
</p>

<div align="center">

![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/rm-dr/copper)
![GitHub Repo stars](https://img.shields.io/github/stars/rm-dr/copper)
![GitHub License](https://img.shields.io/github/license/rm-dr/copper)

# Contributor Docs

</div>

---

## Project structure

[`./copperc`]: ./copperc/
[`./copperd`]: ./copperd/

Copper consists of a React frontend ([`./copperc`]) and a set of backend daemons ([`./copperd`]) written in Rust.

- [`./copperc`] contains no application logic. Its only job is to draw pretty pictures and send requests to the backend.
- [`./copperd`] is a Cargo workspace organized as follows:
  - [`./copperd/bin`](./copperd/bin/): each directory here corresponds to a backend daemon
    - [`./edged`](./copperd/bin/edged/): copper's "edge" server. It provides the api that `copperc` connects to, and is the _only_ backend daemon that is exposed to the internet. `edged` stores all user metadata (account info, pipelines) and manages login sessions.
    - [`./piper`](./copperd/bin/piper/): Copper's pipeline runner. Takes jobs off the queue and processes them. Many instances of piper may be run in parallel.
  - [`./copperd/lib`](./copperd/lib/): utilities shared by backend daemons
  - [`./copperd/nodes`](./copperd/nodes/): node implementations for `piper`

## Environment setup

All dev tools are in [`./dev`](./dev/). To set up a fresh environment, do the following:

- `cd dev; docker compose up -d`
  - This starts all backing services copper needs.
  - Port binds might conflict with other containers, watch the logs.
  - Buckets and databases will be initialized automatically
- In `copperd`, copy `dev.env` to `.env`
  - You should only need to edit this file if you've edited `./dev/docker-compose.yml`
- `cd copperd` and run each of the following in a different terminal:
  - `cargo run --package edged`
  - `cargo run --package piper`
  - Any daemon that has an API provides documentation at `http://localhost:{port}/docs`
- In a new terminal, `cd copperc` and run `EDGED_ADDR="http://localhost:2000" npm run dev` to start the web ui.

# Version control & Releases

## Git history

- Branches are organized as follows:
  - `main`: primary branch. This is always the latest stable version.
    - Merges into `main` always correspond to a versioned release.
    - TODO: make this automatic
    - We never commit to `main` directly.
  - feature branches are branched off of main.
    - The last commit to a version branch should update the version strings in `copperc/package.json` and `copperd/cargo.toml`.
    - These versions should always be identical (TODO: add a test)
    - feature branches are always merged into `main` with a merge commit. Do not fast-forward or squash.
    - feature branches are always merged with a pr.
- Never merge `main` into any other branch. Always rebase.
  - When a branch is merged, it dies. Do not make any more commits, make a new branch.
- Always clean up your commit history (`git rebase -i main`) before merging.
  - commits should represent independent units of work
  - commits should be as large as possible without violating the previous point

## Release checklist

- [ ] Clean up history on the feature branch
- [ ] Add one more commit that updates version strings
- [ ] Merge version branch into main (pull request)
  - The message of the resulting merge commit **must** start with the new version string.
- [ ] Generate and publish release files (TODO: what does this mean?)

# Minor notes

## Logging

Log level rules of thumb:

- `Error`, if something is very broken
- `Warn`, if something wrong but we have a way out
- `Info`, if something happened that a sysadmin might care about.
- `Debug`, somewhere in between
- `Trace`, if this is a minor event we don't care about unless we're debugging a specific problem.

## Docs to write:

- Node api documentation
  - Nodes should be panic-free, returning an error if resources they need vanish.
- Env var docs
