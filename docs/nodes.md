- Document rust api
- Nodes take input even when not ready


Nodes should be panic-free, returning an error when resources they need vanish.
This situation shouldn't cause deadlocks, since datasets manage their own locks.
