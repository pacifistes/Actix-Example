#!/bin/bash
if [ $# -ge 1 ]
then
    if [ $1 = "start" ]
    then
        docker-compose --env-file "$(dirname $0)/../docker/.env" -f "$(dirname $0)/../docker/docker-compose.yml" up -d --build
    elif [ $1 = "stop" ]
    then
        docker-compose --env-file "$(dirname $0)/../docker/.env" -f "$(dirname $0)/../docker/docker-compose.yml" down
    else
        echo "usage: sh docker.sh [start|stop]"
    fi
else
    echo "usage: sh docker.sh [start|stop]"
fi