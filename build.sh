#!/bin/bash

# Create docker directory if it doesn't exist
mkdir -p ./docker

# Remove existing .env file if it exists
rm -f ./docker/.env

# Setting environment variables for docker-compose
echo "# MongoDB Configuration" >> ./docker/.env
echo "MONGODB_USERNAME=root" >> ./docker/.env
echo "MONGODB_PASSWORD=example" >> ./docker/.env
echo "MONGODB_DATABASE=vehicle_booking" >> ./docker/.env

# API Configuration  
echo "API_PORT=8080" >> ./docker/.env

# MongoDB URI for the application
echo "MONGODB_URI=mongodb://root:example@mongo-container:27017" >> ./docker/.env

echo "Starting Docker containers..."
# Docker commands
docker-compose -f ./docker/docker-compose.yml up -d --build

echo "Containers started."
echo "run the command sh scripts/docker.sh start"
echo "then run the command sh scripts/api.sh start"