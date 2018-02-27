#!/usr/bin/env bash

binary=target/release/unzipr
if [ ! -e "$binary" ]; then
    binary=target/debug/unzipr
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
echo "Running test 1"
output=`$binary -l tests/resources/test.zip`
code=$?
expected="test/
test/test.txt"

assert_eq "$expected" "$output"
assert_success_code $code


# Test 2
# Test listing files in nested zip file
echo "Running test 2"
output=`$binary -l tests/resources/test-test.zip test.zip`
code=$?
expected="test/
test/test.txt"

assert_eq "$expected" "$output"
assert_success_code $code


# Test 3
# Test unpacking files in nested zip file
echo "Running test 3"
output=`$binary -p tests/resources/test-test.zip test.zip test/test.txt`
code=$?
expected="Hello"

assert_eq "$expected" "$output"
assert_success_code $code


# Test 4
# Test failure unpacking a txt file
echo "Running test 4"
output=`$binary -l tests/resources/test.txt`
code=$?
expected="File is not a zip file"

assert_eq "$expected" "$output"
assert_failure_code $code


# Test 5
# Test failure unpacking a non-existent file
echo "Running test 5"
output=`$binary -l tests/resources/asdf`
code=$?
expected="Input file does not exist"

assert_eq "$expected" "$output"
assert_failure_code $code


# Test 6
# Test unpacking to current directory
echo "Running test 6"
cur_dir=$(pwd)
test_dir=target/tests/test6
rm -rf $test_dir
mkdir -p $test_dir
cd $test_dir

expected_contents="Hello"
output=`"$cur_dir/$binary" "$cur_dir/tests/resources/test-test.zip" test.zip`
code=$?

contents=`cat test/test.txt`
assert_eq "" "$output"
assert_eq "$expected_contents" "$contents"
assert_success_code $code
cd $cur_dir


# Test 7
# Test unpacking to specific directory
echo "Running test 7"
test_dir=target/tests/test7
rm -rf $test_dir
mkdir -p $test_dir

expected_contents="Hello"
output=`$binary -d $test_dir tests/resources/test-test.zip test.zip`
code=$?

contents=`cat "$test_dir/test/test.txt"`
assert_eq "" "$output"
assert_eq "$expected_contents" "$contents"
assert_success_code $code


# Test 8
# Test unpacking to specific directory containing ..
echo "Running test 8"
test_dir=target/tests/test8
rm -rf $test_dir
mkdir -p "$test_dir/deep"

expected_contents="Hello"
output=`$binary -d "$test_dir/deep/.." tests/resources/test-test.zip test.zip`
code=$?

contents=`cat "$test_dir/test/test.txt"`
assert_eq "" "$output"
assert_eq "$expected_contents" "$contents"
assert_success_code $code


# Test 9
# Test unpack doesn't overwrite files
echo "Running test 9"
test_dir=target/tests/test9
rm -rf $test_dir
mkdir -p $test_dir

expected_contents="Hello"
output=`$binary -d $test_dir tests/resources/test-test.zip test.zip`
output=`$binary -d $test_dir tests/resources/test-test.zip test.zip`
code=$?

assert_eq "Target file already exists" "$output"
assert_failure_code $code


# Test 10
# Test unpacking to dir fails for non-existent file
echo "Running test 10"
test_dir=target/tests/test10
rm -rf $test_dir
mkdir -p $test_dir

output=`$binary -d $test_dir tests/resources/asdf`
code=$?

assert_eq "Input file does not exist" "$output"
assert_failure_code $code

