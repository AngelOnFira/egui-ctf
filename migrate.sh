#!/bin/bash

# Reset the db


echo "Migrating"

sea-orm-cli migrate fresh -u postgres://postgres:postgres@localhost:5432/postgres

echo "Building entity"

sea-orm-cli generate entity \
    -o entity/src/entities \
    --with-serde both \
    -u postgres://postgres:postgres@localhost:5432/postgres
