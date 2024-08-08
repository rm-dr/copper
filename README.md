# TODO

Poor man's issue tracker. Good enough for now, this team isn't very big.

Projects marked with a 📦 are prerequisites for `v0.1.0` release.
The goal is a *minimal* working version: robust, usable, but possibly slow and missing fancy features.


## 📦 CRUD datasets
- [ ] Rename set, class, attr endpoint
- [ ] Loading state for modals

## 📦 Server.toml
- [ ] read blob size

## 📦 Better dataset names
- Store name in db, use idx as fs path?

## 📦 Database migrations
- old dbs should not be destroyed

## 📦 Dataset locks
- [ ] delete dataset while pipeline is running?
  - Jobs should automatically fail (dataset id?)
- [ ] async dataset api?

## 📦 How to fail pipelines?
- e.g, duplicate album art

## 📦 CRUD items
- [ ] Create items by pipeline
  - [ ] Fetch item node should work
  - [ ] Clean up input list & api
    - get inputs from server
- [ ] Hash files when uploading (incremental)
  - make sure uploads don't expire
  - [ ] clean up upload api
  - [ ] move upload logic to `Uploader`
  - [ ] get fragment size from server config
- [ ] UI item CRUD
  - [ ] View table (endless scroll)
  - [ ] Select attrs to show
  - [ ] Search panel (no logic yet)
  - [ ] Sort by attr

## 📦 Authentication
- [ ] Pick auth method & storage
- [ ] Login endpoint
- [ ] Login page
- [ ] CRUD users and groups from ui
  - group permission to create users and set groups
- [ ] Dataset permissions (per group)


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
- [ ] why is ui so slow?
- [ ] onmousedown: check button, catch keyboard input
- [ ] tab all interactables
- [ ] Next cache config
- [ ] Font
- [ ] Panel width. Center, or change page background?
- [ ] Prettier errors from server (markdown? might be overkill.)
- [ ] Fade bottom of all scrolls
  - dataset tree
  - upload queue


## 📦 Better uploads
- [ ] Upload in parallel
- [ ] Warn when closing window if uploading
- [ ] View and edit queue (?)

## 📦 Pipeline editor
- redo serialize/deserialize pipeline spec
- show error when class ref is invalid (if class was deleted)

## 📦 Pipeline argument nodes
- already in upload ui, just need node implementation

## 📦 Daemon cleanup
- [ ] Rename "fragment", "item class", "database", etc (glossary)
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
- [ ] Structs for big returns in dataset api

## 📦 Logging cleanup
- [ ] logging everywhere
- [ ] well-defined log levels

## 📦 UI Cleanup
 - Rename `upload` page
 - Find all console.log
 - Better dataset tree (open/close spacing, background, etc)

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


# Daemon cleanup v2
- [ ] utoipa tags
- [ ] use memmap2 for files
- [ ] One integer type, with options


## Audit log
- [ ] Track logins
- [ ] Track user actions
- [ ] Audit log admin page

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
- [ ] Store `Binary` mime type

## Ui Polish v2
- [ ] Better errors in modals
- [ ] Better `ApiSelector` loading state
- [ ] Status update shouldn't trigger `ApiSelector` update in upload page
- [ ] Reorder attributes & classes
- [ ] Close dataset menu when dropdown closed
---------------------------------------------------------------------

## Faster main db
- mysql?

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

