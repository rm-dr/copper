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
- Refactor api
  - do all struct serializations make sense (MimeType)
  - Clean up "upload" api
  - Clean up "status" api
  - Add "add job" api
  - Redo "pipeline" api (pipeline editor ui first?)
  - move uploader struct

- Use memmap2
- How and when should we load databases?
  - (nice interop with ufod)
  - Better way to define nodes (compatible with standalone ufo)
- Clean up logging
- ufoc error handling

- Clean up pipeline error handling (search for unwrap, assert, and panic)
  - db errors in pipeline run & build
  - detect bad classes when building AddToDataset node
  - elegantly handle duplicate album art (fail pipelines)
    - how about sub-pipelines?
    - always fail unless explicitly told to `None`
- Deadlock detection



### Small tweaks
- Clean up FLAC code with `readvectored`
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

### Database
- Many-attr unique constraint
- Load and check db metadata
- Store mime with binary data
- Database caching
- Async database
- automatic attributes (computed by a pipeline, like hash of album art)


### Pipeline runner
- Remove other pipeline node?
- Improve node parsing
- Rework pipeline errors
- Smarter pipeline scheduler
  - efficient end condition: we don't need to run ALL nodes
  - What is blocking what? (data streams)
  - Hints? (iobound, networkbound, etc)
  - Nodes ask for other nodes (ifnone)
  - Stop reading file when all dependents are done
  - Nodes ask for other nodes (ifnone)
- Warn on disconnected pipeline inputs
- Arrays & foreach (a file could have many covers)
- Discard node---what should we do for sub-pipelines?
  - Transactions?


### Later
- Better name; branding & site
- tui, web ui, server with auth, api
- Docs
  - classes & attrs are immutable (cannot change once made)
  - node deadlocks: buffer blobs even if input not ready
  - Definitions:
    - pipeline & pipelinespec: definition of pipeline
    - runner: manages many jobs
    - job: an instance of one pipeline, possibly with many threads
    - database = blobstore + metadb
    - pipeline nodes should never panic. Return errors instead.
      - Runner should handle panics?
    - Pipelines are one-off runs, NOT stream processors!
    - Multiple file readers to prevent high memory use
  - blobstore, metastore and pipestore do not need to be mutable. They handle locking on their own!
- Fast search (index certain attributes)
- Save pipelines in database
- Web streams as pipeline input
- Continuously-running pipelines
  - pipelines are still one-off runs.
  - Streams get split and `foreach`ed.
- Plain pipeline tui


### Write tests:
- Tiny blob queue sizes
- Big blob queue sizes
- Malformed flac files (many headers, not a flac, too long, etc)
