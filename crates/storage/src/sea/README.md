# sea-orm Dataset backend

Do NOT edit any files in `./entities`.
These are generated from the migration spec using
```bash
sea-orm-cli generate entity -u "sqlite:./test.sqlite?mode=rwc" -o crates/storage/src/sea/entities
```

Make sure you've created a database with your new migration spec first.
