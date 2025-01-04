#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

#readonly TARGET_HOST=trevor@cncpi.lan
readonly TARGET_HOST=trevor@192.168.0.233
readonly TARGET_PATH=/home/trevor/cnc_driver/cnc_driver.elf
readonly SOURCE_PATH=./target/avr-atmega2560/release/cnc_driver.elf

cargo build --release
scp -O -r ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} "ravedude mega2560 ${TARGET_PATH}"
