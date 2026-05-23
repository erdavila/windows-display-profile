#!/bin/bash
set -ex

DATA_DIR=display-profile/data
CS_DIR=$DATA_DIR/clone-squared

function prepare() {
  local DIR="$1"
  mkdir -p $CS_DIR/$DIR
  rm -f dump-*.json
}

function collect() {
  local DIR="$1"
  for DUMP in dump-*DisplayConfig-*.json ; do
    display-profile/data/aggregate-jsons.sh $DUMP > $CS_DIR/$DIR/${DUMP/%.json/-aggregated.json}
  done
  mv dump-*.json $CS_DIR/$DIR
}

case "$1" in
  1)
    prepare 1-pre
    just profile-dump apply $DATA_DIR/profiles/AOC+Dell-extended/profile.json
    collect 1-pre
    ;;

  2)
    prepare 2-apply
    cargo run -p windows-ccd-example apply
    collect 2-apply
    ;;

  3)
    prepare 3-revert
    # This will fail :-(
    ! just profile-dump apply $DATA_DIR/profiles/AOC+Dell-extended/profile.json
    collect 3-revert
    ;;

  *)
    echo "Invalid step" >&2
    exit 1
    ;;
esac
