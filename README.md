# Notes
Eventually, consolidate these in docs
- If a dataset, class, or attr is deleted under a running pipe, that pipe should normally fail.
- When nodes are run, assume they got all input

- Document rust api
- Nodes take input even when not ready


Nodes should be panic-free, returning an error when resources they need vanish.
This situation shouldn't cause deadlocks, since datasets manage their own locks.

- What log level should I use?
  - `Error`, if something is wrong and we can't continue
  - `Warn`, if something wrong and we're ok
  - `Info`, if something happened that a sysadmin might care about.
    - *Note: the default log level for all internal modules is `Info`*
  - `Debug`, somewhere in between
  - `Trace`, if this is a minor event we don't care about unless we're debugging a specific problem.

- Where should I log X?
  - Actions should be logged where they happen (e.g, not in api, in the fn that actually does it)
  - Errors should be logged where they are HANDLED, not where they occur
    - (maybe change later?)
