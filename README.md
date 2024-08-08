# Notes
Eventually, consolidate these in docs
- If a dataset, class, or attr is deleted under a running pipe, that pipe should normally fail.
Nodes should be panic-free, returning an error when resources they need vanish. This situation
also shouldn't cause deadlocks, since datasets manage their own locks.
- Pipeline nodes that need to call async functions should just `block_on` them. Nodes are run in
a threadpool, and are thus inherintly async. This isn't ideal, though, we might want to fix this
later (part of writing a better scheduler).




# TODO

Poor man's issue tracker. Good enough for now, this team isn't very big.

Projects marked with a 📦 are prerequisites for `v0.1.0` release.
The goal is a *minimal* working version: robust, usable, but possibly slow and missing fancy features.

## 📦 Fixes
- [ ] Server deadlocks with two parallel clients

## 📦 API endpoints
- [ ] Rename users & groups
- [ ] Change user password
- [ ] Change self password
- [ ] Rename set, class, attrs

## 📦 CRUD items
- [ ] Infinite scroll when page doesn't fill view
- [ ] Manage ui items
  - [ ] Delete items
  - [ ] Sort by attrs
  - [ ] Class exemplar
  - [ ] Multi-select and edit
  - [ ] Better reference panel
- [ ] Edit ui items
  - [ ] Edit items even if they differ
  - [ ] Show "changed" indicator
  - [ ] "Commit" button


## 📦 Database migrations
- old dbs should not be destroyed

## 📦 Audiofile library
- [ ] Tests
  - [ ] Basic read
  - [ ] Striptags integrity check
  - [ ] Malformed file integrity check
    - Out-of-spec, but blocks ok
    - blocks don't align
- [ ] Readvectored
- [ ] FLAC complete implementation
  - [ ] Handle errors
  - [ ] Multiple covers (take first for now)
  - [ ] Cover inside comment
- [ ] IDv3 complete implementation
- [ ] Tag node types

## 📦 UI Cleanup
- [ ] tab all interactables
- [ ] Font
- [ ] Panel width. Center, or change page background?
- [ ] Prettier errors from server (markdown? might be overkill.)
- [ ] Fade bottom of all scrolls (component)
- [ ] Loading state for modals
- [ ] Modal builder
- [ ] Add attr form
- [ ] Sidebar & top bar shadow
- [ ] Icons in attr & dataset dropdown (component)
- [ ] "No items" element in item table doesn't span columns


## 📦 Better uploads
- [ ] Upload in parallel
- [ ] Warn when closing window if uploading
- [ ] View and edit queue (?)

## 📦 Pipeline editor
- redo serialize/deserialize pipeline spec
- show error when class ref is invalid (if class was deleted)

## 📦 Pipeline argument nodes
- already in upload ui, just need node implementation
- Dynamic input panel

## 📦 Daemon cleanup
- [ ] Rename "fragment", "item class", "database", "blob fragment", "pipeline", "job", etc (glossary)
- [ ] clone fewer arcs
- [ ] fix all panics/unwraps
- [ ] Remove petgraph (write cycle detection algo)
- [ ] Log to file (basic)
- [ ] Minor TODOs in code (search all files)
- [ ] clean up dependencies
- [ ] Enum for api errors (consistent response & log message)
- [ ] Check serializations
- [ ] Force nonempty set, attr, class names
- [ ] Error if full db path doesn't exist (no panic)
- [ ] Text vs long text datatypes
- [ ] Deletion could take a while. Will our request time out?
- [ ] Clippy
- [ ] What if we panic when an unfinished blob is dropped? (log)
- [ ] Which locks should be async?
- [ ] No block_on
- [ ] Use sqlx pool
- [ ] Use fewer `Arc<Mutex>`
- [ ] Why is hash length broken
  - [ ] one tostr implementation
- [ ] stream big files in `/items/attr`
- [ ] `std::fs::read` when reading whole file
- [ ] Cleanup all server arch (never block async)

## 📦 Logging cleanup
- [ ] logging everywhere
- [ ] well-defined log levels
- [ ] log locks?

## 📦 UI Cleanup
- Rename `upload` page
- Find all console.log
- Better dataset tree (open/close spacing, background, etc)
- Show running pipeline node count & progress
- Use JS maps instead of objects
- Fix all type definitions
  - Replace `any`
  - Generate api?

## 📦 Config cleanup
- [ ] accept envvars (docker) and toml file?
- [ ] default values for everything
- [ ] configure pipeline runner (n threads, n pipelines)

## 📦 Distribution
- [ ] Docker file & compose
- [ ] `crates.io`
- [ ] Clippy check rule
- [ ] Auto build & test
- [ ] Git config, rules (docs), and rebase

## 📦 Branding
- [ ] Better name
- [ ] Better logo
- [ ] Website (main page & user docs)

## 📦 Dev docs
- [ ] How to make nodes (cmd api & rust api)
  - never panic
- [ ] glossary of terms
- [ ] Finalize node api (traits, cmd later)
- Notes
  - Pipeline = one-off job. No streams!
  - Nodes take input even when not ready

## 📦 Inline documenation
- UI should be usable without a manual
- [ ] Dataset & attribute type descriptions


---------------------------------------------------------------------


## Daemon cleanup v2
- [ ] utoipa tags
- [ ] use memmap2 for files
- [ ] One integer type, with options
- [ ] Generic datasets, other dataset types
- [ ] Helpful error backtraces (anyhow?)

## More async
- [ ] asyncify blobstore
- [ ] asyncify pipestore
- [ ] should pipelinenodetype methods be async?

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

## Queue jobs (TrueNAS-style)
- dataset deletion could take a while. Maybe keep an async task queue?
- or, find a solution to this problem


## Arrays in pipelines
- Some nodes could return multiple elements (music with many covers). How should we handle this?

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

## Better logging
- Different events to different files?


## More nodes:
- [ ] Node spec codegen
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
- [ ] Better errors in modals
- [ ] Better `ApiSelector` loading state
- [ ] Status update shouldn't trigger `ApiSelector` update in upload page
- [ ] Reorder attributes & classes
- [ ] Close dataset menu when dropdown closed
- [ ] Preview panel data on hover
- [ ] Audio player waveform
- [ ] More panel types: video, pdf
---------------------------------------------------------------------

## Expand authentication
- [ ] item ownership (delete what I created)
- [ ] Dataset permissions (per group)
- [ ] Hide datasets/groups (view permission)
- [ ] Clean up permission model (make a lib)


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
- [ ] Save user preference
- [ ] Set primary color for site (admin)
- [ ] Set site message / logo

## Dataset constraints
- [ ] not null
- [ ] unique
- [ ] multi-unique
- [ ] Make sure all these hold on CRUD

## Automatic dataset backups

## Pipes as transactions
- If a pipe fails, a dataset should not have partial state

## Virtual attributes
- Attributes computed by a pipeline, auto-updated on change

