#!/bin/bash

if ! command -v curl &> /dev/null
then
  echo 'curl is not installed'
  exit
fi

if ! command -v jq &> /dev/null
then
  echo 'jq is not installed'
  exit
fi

curl                                                                   \
  -X POST https://overpass-api.de/api/interpreter                      \
  -H 'Accept: application/json'                                        \
  -H 'Content-Type: application/x-www-form-urlencoded; charset=UTF-8'  \
  -o osm-response.json                                                 \
  --data-binary @- <<QUERY
[out:json][timeout:300];
(
  node["payment:bitcoin"="yes"];
  way["payment:bitcoin"="yes"];
  relation["payment:bitcoin"="yes"];
);
out center;
QUERY

if [ $? -ne 0 ]; then
  echo 'Failed to pull places'
  exit 1
fi

jq .elements osm-response.json > osm-places.json

echo "Pulled $(jq length places.json) places"
