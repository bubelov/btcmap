#!/bin/bash

if ! command -v sqlite3 &> /dev/null
then
  echo "sqlite3 is not installed"
  exit
fi

if ! [ -f schema.sql ]; then
  echo 'schema.sql is missing'
  exit
fi

sqlite3 btcmap.db < schema.sql

if [ $? -eq 0 ]; then
  echo 'Created btcmap.db'
fi

