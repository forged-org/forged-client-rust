# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.4.0 - 2024-02-08

### Added
* Added a new `Client::binary_part()` API to download binary parts to the local machine and get
their contents
    * Downloaded contents are cached in $HOME/.forged

## 0.3.0 - 2023-05-05

### Added
* Added a new `Client::blocks()` API to get the blocks for the current run

### Changed
* The `Client` can now be constructed via `Client::default()` to grab the token and
instance URL from the environment automatically.
