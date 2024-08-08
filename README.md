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
- async binary readers
- handle channel errors (Pending when full?)
- args to node one by one
- sanely handle duplicate album art
- Clean up all error handling (search for unwrap, assert, and panic)
- clean up paths (pub use)

### Later:
- Better name
- tui, web ui
- async pipeline runners
- Docker container
- Docs
- limit channel size
- Dynamic node definitions
- Fast search
- Discard node---what should we do for sub-pipelines?
  - Transactions?
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
- Clean pipeline prep() (catch as many errors as possible before running)
- Web streams as pipeline input
- Continuously-running pipelines
