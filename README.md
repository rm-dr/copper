# UFO: The Universal File Organizer


UFO can replace:
- [Paperless] (and similar DMS)
- [Calibre]
- [Beets] (and similar music managers)
- [Picard], [EasyTag] (no need to manually tag music)


UFO's goal is to be "[Paperless] for everything," with...
- Flexible, fast, and automatable data processing via pipelines
  - Data ingest, processing, and export
- Fast search & metadata editing
- A pretty web ui


[Paperless]: https://docs.paperless-ngx.com
[Calibre]: https://calibre-ebook.com
[Beets]: https://beets.io
[Picard]: https://picard.musicbrainz.org/
[EasyTag]: https://wiki.gnome.org/Apps/EasyTAG



## TODO:

### Current:
- elegantly handle duplicate album art

- no channel for take_input
- pipeline node logging
- handle channel errors (Pending when full?)
- limit channel size
- rework flac methods, better errors (when incomplete data)
- flac strip stream (internal buffer, read & write?)

- Clean up all error handling (search for unwrap, assert, and panic)
  - db errors in pipeline run & build
  - detect bad classes when building AddToDataset node



### Small tweaks
- Add nodes:
  - Audio metadata: bit rate, length, sample rate, etc
  - Strip spaces, regex
  - external commands
- Add datatypes:
  - enum
  - multi-enum
  - date
- Helpful pipeline parse errors:
  - deserialize reference from name
- Faster node inputs() and outputs()
  - Fewer db hits (solve by caching?)
- Clean up dependencies
- Remove petgraph
  - Write toposort algo, provide whole cycle in errors

### Dataset
- Load and check db metadata
- Clean up blobstore
- Store mime with binary data
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
- Better name; branding & site
- tui, web ui, server with auth, api
- Docs
  - classes & attrs are immutable (cannot change once made)
- Fast search (index certain attributes)
- Save pipelines in database
- Web streams as pipeline input
- Continuously-running pipelines
- Plain pipeline tui
