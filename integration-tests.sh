#!/usr/bin/env bash

binary=target/debug/unzipr
if [ ! -e "$binary" ]; then
    binary=target/release/unzipr
fi

function assert_code() {
    expected=$1
    code=$2
    if [ "$code" -ne "$expected" ]; then
        echo "Exit code $code is not 0"
        exit 1;
    fi
}
function assert_eq() {
    expected=$1
    output=$2
    if [ "$output" != "$expected" ]; then
        echo "'$output' does not equal '$expected'"
        exit 1
    fi
}
function assert_success_code() {
    assert_code 0 $1
}
function assert_failure_code() {
    assert_code 1 $1
}

# Test 1
# Test listing files in a zip file
output=`$binary -l tests/resources/test.zip`
code=$?
expected="test/
test/test.txt"

assert_eq "$expected" "$output"
assert_success_code $code

# Test 2
# Test listing files in nested zip file
output=`$binary -l tests/resources/test-test.zip test.zip`
code=$?
expected="test/
test/test.txt"

assert_eq "$expected" "$output"
assert_success_code $code

# Test 3
# Test unpacking files in nested zip file
output=`$binary -p tests/resources/test-test.zip test.zip test/test.txt`
code=$?
expected="Hello"

assert_eq "$expected" "$output"
assert_success_code $code

# Test 4
# Test failure unpacking a txt file
output=`$binary -l tests/resources/test.txt`
code=$?
expected="File is not a zip file"

assert_eq "$expected" "$output"
assert_failure_code $code

# Test 5
# Test failure unpacking a non-existent file
output=`$binary -l tests/resources/asdf`
code=$?
expected="Input file does not exist"

assert_eq "$expected" "$output"
assert_failure_code $code
