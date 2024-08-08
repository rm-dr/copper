# Custom FLAC test files

These are flac files created specifically for Copper, and test cases that the flac test toolkit doesn't cover.
Most of these are modified copies of files in `flac_subset`, `flac_faulty`, or `flac_uncommon`


## Manifest

- `01 - many images.flac`: This is `flac_subset/50` with additional images from `56`, `57`, `58`, and `59`, in that order.
  - Image 0: from file `50`, type is `3`, description is empty.
  - Image 1: from file `56`, type is `17`, description is `lorem`.
  - Image 2: from file `57`, type is `2`, description is `ipsum`.
  - Image 3: from file `58`, type is `12`, description is `dolor`.
  - Image 4: from file `59`, type is `4`, description is `est`.
- `02 - picture in vorbis comment.flac`: This is `flac_subset/57`, but with the image stored inside a vorbis `METADATA_BLOCK_PICTURE` comment instead of a proper flac picture metablock.
- `03 - faulty picture in vorbis comment.flac`: This is `02`, but with a corrupt picture.
