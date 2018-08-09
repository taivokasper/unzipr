# Unzipr
[![Travis build status](https://img.shields.io/travis/taivokasper/unzipr.svg)](https://travis-ci.org/taivokasper/unzipr)
[![GitHub latest release](https://img.shields.io/github/release/taivokasper/unzipr.svg)](https://github.com/taivokasper/unzipr/releases/latest)
[![Dependencies up-to-date](https://img.shields.io/librariesio/github/taivokasper/unzipr.svg)](https://github.com/taivokasper/unzipr/blob/master/Cargo.toml)

[![Latest version downloads](https://img.shields.io/github/downloads/taivokasper/unzipr/latest/total.svg)](https://github.com/taivokasper/unzipr/releases/latest)
[![Total downloads](https://img.shields.io/github/downloads/taivokasper/unzipr/total.svg)](https://github.com/taivokasper/unzipr/releases)

[![Commit activity the past year](https://img.shields.io/github/commit-activity/y/taivokasper/unzipr.svg)](https://github.com/taivokasper/unzipr/commits)
[![Contributors](https://img.shields.io/github/contributors/taivokasper/unzipr.svg)](https://github.com/taivokasper/unzipr/graphs/contributors)

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
