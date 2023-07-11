migrate:
	#!/bin/bash

	@ Reset the db

	echo "Migrating"

	sea-orm-cli migrate fresh -u postgres://postgres:postgres@localhost:5432/postgres

	echo "Building entity"

	sea-orm-cli generate entity \
		-o entity/src/entities \
		--with-serde both \
		-u postgres://postgres:postgres@localhost:5432/postgres

make-migration:
	sea-orm-cli migrate \
		generate \
		-u postgres://postgres:postgres@localhost:5432/postgres \
		new_migration

deploy:
	nomad job restart \
		-address=http://localhost:4646 \
		-group ctf-backend \
		-task actix-backend \
		ctf-dashboard

	nomad job restart \
		-address=http://localhost:4646 \
		-group ctf-discord-bot \
		-task serenity-bot \
		ctf-dashboard

check:
	#!/usr/bin/env bash
	# This scripts runs various CI-like checks in a convenient way.
	set -eux

	cargo check --workspace --all-targets
	cargo check --workspace --all-features --lib --target wasm32-unknown-unknown
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --all-features --  -D warnings -W clippy::all
	cargo test --workspace --all-targets --all-features
	cargo test --workspace --doc
	trunk build

tidy:
	cargo fix --workspace --allow-dirty --allow-staged
	cargo clippy --fix --workspace --allow-dirty --allow-staged
	cargo fmt

frontend:
	cd frontend \
	&& trunk serve

backend:
	cd backend \
	&& cargo run

set dotenv-load

discord:
	cd discord-bot \
	&& cargo run