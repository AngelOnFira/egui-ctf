#!/bin/bash

sea-orm-cli migrate -u sqlite://file.db

rm -rf entity/src/entities/
sea-orm-cli generate entity -o entity/src/entities \
    --with-serde both
