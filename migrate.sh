#!/bin/bash

# Reset the db
rm -rf file.db
sqlite3 file.db "VACUUM;"

sea-orm-cli migrate -u sqlite://file.db

rm -rf entity/src/entities/
sea-orm-cli generate entity -o entity/src/entities \
    --with-serde both
