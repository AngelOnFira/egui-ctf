#!/bin/bash

# Reset the db
rm -rf file.db
sqlite3 file.db "VACUUM;"

echo "Migrating"

sea-orm-cli migrate -u sqlite://file.db

echo "Building entity"

rm -rf entity/src/entities/
sea-orm-cli generate entity \
    -o entity/src/entities \
    --with-serde both
