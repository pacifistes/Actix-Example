#!/bin/bash
if [ $# -ge 1 ]
then
    if [ $1 = "run" ]
    then
        docker exec -w /usr/src/project/Actix-Example/vehicle-api -t glanceable-container bash -c 'cargo run --release'
    elif [ $1 = "build" ]
    then
        docker exec -w /usr/src/project/Actix-Example/vehicle-api -t glanceable-container bash -c 'cargo build --release'
    elif [ $1 = "test" ]
    then
        docker exec -w /usr/src/project/Actix-Example/vehicle-api -t glanceable-container bash -c 'cargo test -- --nocapture'
    elif [ $1 = "fmt" ]
    then
        docker exec -w /usr/src/project/Actix-Example/vehicle-api -t glanceable-container bash -c 'cargo fmt'
    elif [ $1 = "clippy" ]
    then
        docker exec -w /usr/src/project/Actix-Example/vehicle-api -t glanceable-container bash -c 'cargo clippy'
    elif [ $1 = "check" ]
    then
        docker exec -w /usr/src/project/Actix-Example/vehicle-api -t glanceable-container bash -c 'cargo check'
    elif [ $1 = "doc" ]
    then
        docker exec -w /usr/src/project/Actix-Example/vehicle-api -t glanceable-container bash -c 'cargo doc --no-deps --release'
    else
        echo "usage: sh api.sh [run|build|test|fmt|clippy|check|doc]"
    fi

else
    echo "usage: sh api.sh [run|build|test|fmt|clippy|check|doc]"
fi