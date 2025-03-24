#!/bin/bash

TIMESTAMP=$(date +"%Y-%m-%d_%H-%M")
mkdir -p backups
# shellcheck disable=SC2046
docker exec $(docker ps --filter "name=db" --format "{{.Names") pg_dump -U user -d games_db > backups/games_db_$TIMESTAMP.sql