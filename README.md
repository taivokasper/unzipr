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

# Installing on MacOs with Homebrew
```bash
brew tap taivokasper/utilities
brew install unzipr
```

# Building
```bash
cargo build --release
```
