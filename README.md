# Notes
Eventually, consolidate these in docs
- If a dataset, class, or attr is deleted under a running pipe, that pipe should normally fail.

# TODO

The list below is a minimal issue tracker.

Projects marked with a 📦 are prerequisites for `v0.1.0` release.
The goal is a minimal working version: robust and usable, but possibly slow and missing fancy features.

## 📦 Daemon cleanup
- [ ] remove panics/unwraps
- [ ] Minor TODOs in code (search all files)
- [ ] stream big files in `/items/attr`
- [ ] Remove allow dead code (after implementing features)

## 📦 Fixes
- [ ] Server deadlocks with two parallel clients
- [ ] Backup db when migrating
- [ ] Do not try to load when item list is scrolled to bottom

## 📦 View items
- [ ] Delete items
- [ ] Sort by attrs

## 📦 Audiofile library
- [ ] IDv3 complete implementation

## 📦 Pipeline ui
- Redo input ui for new input arch
- Pipeline editor
- Node docs (inside ui)

## 📦 UI Cleanup
- [ ] Image preview on hover in table
- [ ] Image placeholder while loading
- Show running pipeline node count & progress
- [ ] Remove zustand
- [ ] Server components
- [ ] React query

## 📦 Upload page
- Rename (and redesign) `upload` page
  - Panel for every input?
- [ ] Upload in parallel
- [ ] Warn when closing window if uploading
- [ ] View and edit queue (?)
- [ ] Show all errors in ui
- [ ] Improve "new file" check
- [ ] redo input exemplars

## 📦 Distribution
- [ ] Docker file & compose
- [ ] `crates.io`
- [ ] CI:
  - [ ] cargo build
  - [ ] docker build

## 📦 Branding
- [ ] Better iconography & font
- [ ] README
- Dev docs

## 📦 Inline documentation
- UI should be usable without a manual (info hovers)
- [ ] Dataset & attribute type descriptions


---------------------------------------------------------------------

## Edit items
- [ ] Edit items even if they differ
- [ ] Show "changed" indicator
- [ ] "Commit" button
- [ ] Edit panel items

## Daemon cleanup v2
- [ ] utoipa tags
- [ ] use memmap2 for files
- [ ] Generic datasets, other dataset types
- [ ] Remove petgraph (write cycle detection algo)
- [ ] "stay logged in" checkbox

## Security
- [ ] Rate-limit api
- [ ] Log request ips
- [ ] Block ips

## Audit log
- [ ] Track logins
- [ ] Track user actions
- [ ] Audit log admin page
- [ ] Impersonate users
- [ ] Show active sessions

## Dashboard (UI home page)
- [ ] Show counts & sizes
- [ ] Job history
- [ ] Job history graph
- [ ] Show dataset metadata in dataset page
  - size, item count

## Dataset caching
- [ ] Cache built pipelines
- [ ] Cache common metastore gets

## CRUD Jobs
- [ ] Clean up pipeline error handling
- [ ] Show job log in upload page
- [ ] Job log page:
  - Failed jobs with message
  - Input exemplars
  - Job log expires after `n` hours
  - show `created_at` in job log
  - [ ] filter and sort jobs
- [ ] Cancel pending and running jobs

## Queue tasks (TrueNAS-style)
- dataset deletion could take a while
- attr deletion could take a while
- Keep a background task queue for long-running jobs so ui doesn't hang.

## Arrays in pipelines
- Some nodes could return multiple elements (music with many covers). How should we handle this?
- Flac node: extract all covers and send array

## "other pipeline" node
- append to back of job queue, no output

## export jobs
- perodic (backup)
- on demand
- download all items where...
- Run export pipeline on subset of items

## Search items
- [ ] Configure search index on attributes
- search should be fast and robust, even on *huge* datasets

## Better deletion
- show item count/attr count/size

## Better pipeline scheduler
- [ ] better end condition (only effectful nodes?)
- [ ] don't run nodes if not necessary (`ifnone`)
  - nodes ask for nodes?
- [ ] do we need `after` edges?
- [ ] don't read file if no deps
- [ ] we used to be able to use multiple file readers to save memory.
  - now what?
- Stop producing binary if next node is done

## More nodes:
- [ ] node tests
- [ ] hash additional types
- [ ] external command (for user plugins)
  - ollama, whisper
- [ ] email
- string manipulation
  - [ ] strip
  - [ ] regex replace
  - [ ] regex search
  - [ ] lower/upper
- [ ] audio file metadata (bit rate, etc)
- type conversion
  - [ ] number to string

## More storage types:
- [ ] Enums
- [ ] Multi-enums
- [ ] How to store playlists?
  - in their own class, with a list of refs?
- [ ] Date
- [ ] Time

## Ui Polish v2
- [ ] Better errors in modals (format on server)
- [ ] Better `ApiSelector` loading state
- [ ] Status update shouldn't trigger `ApiSelector` update in upload page
- [ ] Reorder attributes
  - fix index on delete
  - classes always alphabetical
- [ ] Preview panel data on hover
- [ ] Better audio player (center, fill, waveform)
- [ ] More panel types: video, pdf
- [ ] Go to item in reference panel (how?)
- [ ] Reference panel backlinks
- [ ] Icons in attr & dataset dropdown
- [ ] Infinite scroll when item table doesn't fill view
  - just use a big enough page size?
- [ ] Show menu on right-click in trees
- [ ] tab all interactables
- [ ] Fade bottom of all scrolls (component? overscroll?)
- Right-click menus


---------------------------------------------------------------------

## Expand authentication
- [ ] item ownership (delete what I created)
- [ ] Dataset permissions (per group)
- [ ] Hide datasets/groups (view permission)
- [ ] Clean up permission model (make a lib)
- [ ] Disable no-permission buttons in group tree (ui)

## Other datasets
- mysql + ?
- object store?
- No blobs at all (with fast db backend)
- Each dataset has its own types?

## Hash blobs
- integrity check?
- deduplicate

## Pipeline builder
- an invalid pipeline should deserialize, but should not build
- (gives user opportunity to fix errors)
- [ ] Better type checking
  - [ ] `string | null` types
  - [ ] Catch as many errors as possible when building pipeline
- [ ] Warnings (disconnected inputs)

## Read-only "views" into data?
- allow other apps to use our db (jellyfin, syncthing, etc)

## Tasks
- Trigger jobs automatically on some event
- [ ] email
- [ ] ytdl
- [ ] rss
- [ ] filesystem?
- [ ] public / apikey POST
- [ ] Task log

## UI config
- [ ] Light/dark theme
- [ ] Set primary color for site (admin)
- [ ] Set site message / logo

## Dataset constraints
- [ ] not null
- [ ] unique
- [ ] multi-unique
- [ ] Make sure all these hold on CRUD

## Automatic dataset backups

## Pipelines as transactions
- If a pipe fails, a dataset should not have partial state
- how should we clean up?

## Virtual attributes
- Attributes computed by a pipeline, auto-updated on change

## Audiofile library
- Linked images
- Skip bad blocks (don't reject whole file)
- Validations
  - (vorbis) There may be only one picture of type type 1 and 2 in a file
  - (flac) Multiple vorbis comment blocks are an error
  - (flac) enforce length limits for all ints
  - (flac) enforce block length limits
  - (flac) Many metadata blocks
  - (flac) Many streaminfo blocks
  - tests
- [ ] Early exit if we don't need audio data
