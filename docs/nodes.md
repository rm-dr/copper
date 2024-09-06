- Document rust api
- Nodes take input even when not ready


Nodes should be panic-free, returning an error when resources they need vanish.
This situation shouldn't cause deadlocks, since datasets manage their own locks.

- Pipeline nodes that need to call async functions should just `block_on` them. Nodes are run in
a threadpool, and are thus inherently async. This isn't ideal, though, we might want to fix this
later (part of writing a better scheduler).
