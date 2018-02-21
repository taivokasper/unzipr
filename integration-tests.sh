#!/usr/bin/env bash
set -e

binary=target/debug/unzipr
if [ ! -e "$binary" ]; then
    binary=target/release/unzipr
fi

# Test 1
# Test listing files in a zip file
output=`$binary -l tests/resources/test.zip`
expected="test/
test/test.txt"

if [ "$output" != "$expected" ]; then
    echo "'$output' does not equal '$expected'"
    exit 1
fi

# Test 2
# Test listing files in nested zip file
output=`$binary -l tests/resources/test-test.zip test.zip`
expected="test/
test/test.txt"

if [ "$output" != "$expected" ]; then
echo "'$output' does not equal '$expected'"
    exit 1
fi

# Test 3
# Test unpacking files in nested zip file
output=`$binary -p tests/resources/test-test.zip test.zip test/test.txt`
expected="Hello"

if [ "$output" != "$expected" ]; then
echo "'$output' does not equal '$expected'"
    exit 1
fi