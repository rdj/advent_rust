#!/usr/bin/env bash

ME=$(basename "$0")
ME_NOEXT="${ME%%.*}"
MYPATH=$(dirname "$0")
HEADERS_FILE="${MYPATH}/${ME_NOEXT}.txt"
DAY=$1
YEAR=${2:-$(date "+%Y")}

if [ "${DAY}" == "" ]; then
    echo "usage: ${ME} day [year]" 1>&2
    exit 1
fi

if [ ! -f "${HEADERS_FILE}" ]; then
    echo "missing headers file at ${HEADERS_FILE}"
    echo "should contain your adventofcode.com session cookie in this format:"
    echo "Cookie: session=0123456789abcdef..."
    exit 1
fi

curl -H "@${HEADERS_FILE}" "https://adventofcode.com/${YEAR}/day/${DAY}/input"
