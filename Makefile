SHELL := /bin/bash
init:
	docker-compose -f docker-compose.init.yaml up -d || exit 1
	echo "Waiting for 20 seconds"
	sleep 20
	echo "executing sql file to mysql"
	docker exec -i mysql /usr/bin/mysql < misc/schema/iot.sql && echo "Database structure built"
	echo "Docker bootstrap complete"
	docker-compose -f docker-compose.init.yaml down --remove-orphans || exit 1;
build:
	cargo build --release
redeploy:
	docker-compose down --remove-orphans || exit 1
	cargo build --release
	docker-compose --compatibility up -d --build
	docker-compose logs -f

deploy_base:
	docker-compose -f docker-compose-base.yaml up -d
reload:
	docker-compose down || exit 1
	docker-compose up -d
	docker-compose logs -f