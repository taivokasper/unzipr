# Unzipr
[![Build Status](https://travis-ci.org/taivokasper/unzipr.svg?branch=master)](https://travis-ci.org/taivokasper/unzipr)
[![release](http://github-release-version.herokuapp.com/github/taivokasper/unzipr/release.svg?style=flat)](https://github.com/taivokasper/unzipr/releases/latest)

A command line utility for in-memory listing and unpacking files from nested zip files.

# Usage
* Listing files from a zip file in a zip file
```bash
$ unzipr -l test-zip-of-zip.zip test.zip
test/
test/test.txt
```
* Unpacking files from a nested zip file to standard out
```bash
$ unzipr -p test-zip-of-zip.zip test.zip test/test.txt
Hello World!
```
* Unpacking files from a nested zip file to current working directory
```bash
$ unzipr test-zip-of-zip.zip test.zip
```
* Unpacking files from a nested zip file to specific directory
```bash
$ unzipr -d /tmp/target test-zip-of-zip.zip test.zip
```
# Installing
## MacOs with Homebrew
```bash
brew tap taivokasper/utilities
brew install unzipr
```

## 64-bit Linux
* Download the latest release from [here](https://github.com/taivokasper/unzipr/releases)
* Add the binary to path

## Windows
* Build from source :(

# Building
```bash
cargo build --release
```

# Testing
```bash
cargo test
cargo build
./integration-tests.sh
```

# Contributing
Issues and/or pull requests are welcome!
