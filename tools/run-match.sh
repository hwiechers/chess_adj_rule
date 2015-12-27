#!/usr/bin/env bash

set -eu

(which cutechess-cli &>/dev/null) || {
    echo "cutechess-cli command not found" >&2
    exit 1
}

(which stockfish &>/dev/null) || {
    echo "stockfish command not found" >&2
    exit 1
}

usage_exit() {
    echo "usage: ${0##*/}  [-r <rounds>] <pgn_out>" >&2
    exit 1
}

while getopts "r:" opt
do
    case $opt in
    r) rounds=$OPTARG;;
    *) usage_exit;;
    esac
done

shift $(( OPTIND - 1 ))

if [ $# -gt 1 ]; then
    echo "Too many args. (Remember to put options first.)" >&2;
    usage_exit;
fi

if [ $# -eq 0 ]; then
    echo "<pgn_out> not supplied" >&2;
    usage_exit;
fi

if [ -e $1 ]; then
    echo "Out file '$1' already exists" >&2;
    exit 1;
fi

scriptpath=$(dirname $0)

cutechess-cli -rounds ${rounds:-10} \
-openings file=$scriptpath/2moves_v1.pgn order=random \
-engine name=stockfish1 cmd=stockfish option.Hash=128 \
-engine name=stockfish2 cmd=stockfish option.Hash=128 \
-each proto=uci tc=9.63+0.03 option.Threads=1 \
-pgnout $1 -concurrency 1;
