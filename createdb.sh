#!/bin/bash

sqlite3 btcmap.db < schema.sql

echo 'Created btcmap.db'
