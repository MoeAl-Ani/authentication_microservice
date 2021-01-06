SHELL := /bin/bash

build:
	cargo build --release
redeploy:
	docker-compose down || exit 1
	cargo build --release
	docker-compose --compatibility up -d --build
	docker-compose logs -f

reload:
	docker-compose down || exit 1
	docker-compose up -d
	docker-compose logs -f