#!/usr/bin/env bash

set -eu

if [ $# -ne 1 ]; then
    echo "usage: ${0##*/} <pgn_file>" >&2
    exit 1
fi

grep -oh "[0-9]*\.[0-9]*s" $1 |
sed 's/s$//' |
paste -s -d+ |
bc
