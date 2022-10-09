#!/usr/bin/env bash

ME=$(basename "$0")
MYPATH=$(dirname "$0")
DAY=$1
#YEAR=${2:-$(date "+%Y")}
YEAR=${2:-2019}

if [ "${DAY}" == "" ]; then
    echo "usage: ${ME} day [year]" 1>&2
    exit 1
fi

DAY_2D=$(printf '%02d' ${DAY})

PROJECT="aoc-${YEAR}-${DAY_2D}"

cd "${MYPATH}"
cargo new --lib "aoc-${YEAR}-${DAY_2D}"
cp "${MYPATH}/lib.rs" "${PROJECT}/src/lib.rs"
./aoc-input.sh "${DAY}" "${YEAR}" > "${PROJECT}/input.txt"
emacsclient --no-wait "${PROJECT}/src/lib.rs"


