# UFO: The Universal File Organizer


UFO can replace:
- [Paperless] (and similar DMS)
- [Calibre]
- [Beets] (and similar music managers)
- [Picard], [EasyTag] (no need to manually tag music)


UFO's goal is to be "[Paperless] for everything," with...
- Flexible, fast, and automable data processing (via pipelines)
  - Data ingest, processing, and export
- Fast search & metadata editing
- A pretty web ui
- Easy integration into an authenticated server


[Paperless]: https://docs.paperless-ngx.com
[Calibre]: https://calibre-ebook.com
[Beets]: https://beets.io
[Picard]: https://picard.musicbrainz.org/
[EasyTag]: https://wiki.gnome.org/Apps/EasyTAG



## TODO:

### Current:
- Better db index names

- Load and check db metadata
- Options for hash, reference
  - Better type checking: take (m)any types as input?
- sanely handle duplicate album art

- async binary readers
  - args to node one by one
  - handle channel errors (Pending when full?)
  - limit channel size
- Clean up all error handling (search for unwrap, assert, and panic)
  - db errors in pipeline run & build
  - detect bad classes?
- clean up paths (pub use)


### Later:
- Store big files on fs, not in db
  - Incremental write to storage file
  - Configurable path
  - Configurable backend: fs, object, etc
  - Store mime with binary data
  - upload large files incrementally
- Add datatypes:
  - enum
  - multi-enum
  - date
- Helpful pipeline parse errors
- Smarter pipeline scheduler
  - efficient end condition: we don't need to run ALL nodes
  - What is blocking what? (data streams)
- Warn on disconnected pipeline inputs?
- Detect unused nodes when building?
- Dataset caching
- Better name; branding & site
- tui, web ui
- Docker container
- Docs
  - classes & attrs are immutable (cannot change once made)
- Clean up dependencies
- Async database
- Fast search (index certain attributes)
- Discard node---what should we do for sub-pipelines?
  - Transactions?
- Arrays & foreach (a file could have many covers)
- Add nodes:
  - Audio metadata: bit rate, length, sample rate, etc
  - Strip spaces, regex
  - external commands
- Add attribute propeties:
  - automatic (computed by a pipeline, like hash of album art)
- Save pipelines in database
- Remove petgraph
  - Write toposort algo, provide whole cycle in errors
- Web streams as pipeline input
- Continuously-running pipelines
