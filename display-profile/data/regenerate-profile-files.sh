#!/bin/bash
if [ -z "$1" ] ; then
  echo "Missing profile parameter" >&2
  exit 1
fi

set -ex

PROFILE=$1
PROFILE_DIR=display-profile/data/profiles/$PROFILE

rm -f $PROFILE_DIR/*
just profile-dump save $PROFILE_DIR/profile.json
just profile-dump validate $PROFILE_DIR/profile.json
display-profile/data/aggregate-jsons.sh dump-QueryDisplayConfig-ALL_PATHS.json > $PROFILE_DIR/dump-aggregated.json
jj file track --include-ignored $PROFILE_DIR/dump-aggregated.json
