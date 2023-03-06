#!/usr/bin/env bash

SRC_DIR='./6502_65C02_functional_tests/ca65'
OUT_DIR='./bin'

FUNCTIONAL_TEST="$SRC_DIR/6502_functional_test.ca65"
CONFIG="$SRC_DIR/example.cfg"

function build_file() {
    local FILE=$1
    local FILENAME=$(basename $FILE .ca65)
    local OUT_FILE="$OUT_DIR/$FILENAME"
    ca65 -l "$OUT_FILE.lst" -o "$OUT_FILE.o" "$FILE"
    ld65 "$OUT_FILE.o" -o "$OUT_FILE.bin" -m "$OUT_FILE.map" -C "$CONFIG"
    rm "$OUT_FILE.o"
}

build_file $FUNCTIONAL_TEST

FUNCTIONAL_TEST_NO_DECIMAL='./6502_functional_test_no_decimal.ca65'
sed 's/disable_decimal = 0/disable_decimal = 1/' "$FUNCTIONAL_TEST" > $FUNCTIONAL_TEST_NO_DECIMAL
build_file $FUNCTIONAL_TEST_NO_DECIMAL
rm $FUNCTIONAL_TEST_NO_DECIMAL
