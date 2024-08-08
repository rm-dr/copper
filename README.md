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
- Remove pipeline data types
- Options for hash, reference
- Load "new item" pipeline inputs from db
- sanely handle duplicate album art
- Better index names

- async binary readers
  - args to node one by one
  - handle channel errors (Pending when full?)
  - limit channel size
- Clean up all error handling (search for unwrap, assert, and panic)
- clean up paths (pub use)


### Later:
- classes & attrs are immutable (cannot change once made)
- Better name
- tui, web ui
- async pipeline runners
- Docker container
- Docs
- Clean up dependencies
- Async database
- Store big files on fs, not in db
  - Incremental write to storage file
  - Configurable path
- Dynamic node definitions
- Fast search (index certain attributes)
- Discard node---what should we do for sub-pipelines?
  - Transactions?
- Arrays & foreach (a file could have many covers)
- Add nodes:
  - Audio metadata: bit rate, length, sample rate, etc
  - Strip spaces, regex
  - external commands
- Add datatypes:
  - enum
  - multi-enum
  - date
- Add attribute propeties:
  - not null
  - automatic (computed by a pipeline, like hash of album art)
  - how to enforce?
- Save pipelines in database
- Store dataset spec in db?
- Better db backend
- Remove petgraph & move cycle detection
- Web streams as pipeline input
- Continuously-running pipelines
