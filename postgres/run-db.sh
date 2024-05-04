#!/bin/sh

PGDATA=`readlink -f ../target/pgdata`
echo $PGDATA
docker run --rm --name pg-browser-run --user "$(id -u):$(id -g)" -e POSTGRES_PASSWORD=mysecretpassword -v $PGDATA:/var/lib/postgresql/data -p 5432:5432 postgres:15