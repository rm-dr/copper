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
- foreign key datatype
- add output returns foreign key
- clean up runner
- async binary readers
- limit channel size
- handle channel errors (Pending when full?)
- args to node one by one
- sanely handle duplicate album art
- Attach "after" to *output* of sub-pipeline
- Clean up all error handling (search for unwrap, assert, and panic)

### Later:
- Better name
- tui, web ui
- Docker container
- Docs
- Clean up node definitions
- Fast search
- Discard pipelines---what should we do for sub-pipelines?
  - Transactions?
- Inline nodes (strip spaces, hash, etc)
- Arrays & foreach (a file could have many covers)
- Add nodes:
  - Audio metadata: bit rate, length, sample rate, etc
  - Strip spaces, regex
  - Options for hash (many types)
  - external commands
- Add datatypes:
  - enum
  - multi-enum
  - date
  - int/float
  - hash (md5,sha,etc)
- Add attribute propeties:
  - not null
  - automatic (computed by a pipeline, like hash of album art)
- Save pipelines in database
- Store dataset spec in db?
- Better db backend
- Remove petgraph & move cycle detection
- Clean pipeline prep() (catch as much as possible before running)
- Standalone pipelines
  - Web streams as pipeline input
  - Continuously-running pipelines
