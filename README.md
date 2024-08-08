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
- Ifnone general type

- Load and check db metadata
- sanely handle duplicate album art

- async binary readers
  - args to node one by one
  - handle channel errors (Pending when full?)
  - limit channel size
- Clean up all error handling (search for unwrap, assert, and panic)
  - db errors in pipeline run & build
  - detect bad classes?



### Small tweaks
- Add nodes:
  - Audio metadata: bit rate, length, sample rate, etc
  - Strip spaces, regex
  - external commands
- Add datatypes:
  - enum
  - multi-enum
  - date
- Faster node inputs() and outputs()
  - Fewer db hits (solve by caching?)
- Clean up dependencies
- Remove petgraph
  - Write toposort algo, provide whole cycle in errors

### Dataset
- Store big files on fs, not in db
  - Incremental write to storage file
  - Configurable path
  - Configurable backend: fs, object, etc
  - Store mime with binary data
  - upload large files incrementally
- Dataset caching
- Async database
- automatic attributes (computed by a pipeline, like hash of album art)


### Pipeline runner
- Rework pipeline errors
- Smarter pipeline scheduler
  - efficient end condition: we don't need to run ALL nodes
  - What is blocking what? (data streams)
- Warn on disconnected pipeline inputs
- Detect unused nodes when building
- Arrays & foreach (a file could have many covers)
- Discard node---what should we do for sub-pipelines?
  - Transactions?


### Later
- Helpful pipeline parse errors
- Better name; branding & site
- tui, web ui, server with auth, api
- Docs
  - classes & attrs are immutable (cannot change once made)
- Fast search (index certain attributes)
- Save pipelines in database
- Web streams as pipeline input
- Continuously-running pipelines
- Plain pipeline tui
