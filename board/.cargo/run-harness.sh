#!/bin/bash

if [[ -v TARGET_HOST ]]; then
  TARGET_IDENTITY="keys/$TARGET_HOST"
  scp -i "$TARGET_IDENTITY" "$1" "$TARGET_HOST:/tmp/"
  EXECUTABLE_NAME="/tmp/${1##*/}"
  ssh -i "$TARGET_IDENTITY" "$TARGET_HOST" -- ".cargo/bin/probe-rs run --chip RP2040 '$EXECUTABLE_NAME'"
else
  probe-rs run --chip RP2040 "$1"
fi
