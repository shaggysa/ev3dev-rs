# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-02-24

### Fixed

- Fixed many doc examples to properly use async functions

## [0.3.0] - 2026-02-24

### Changed

- Changed all sensor and motor functions to be async

### Fixed

- Fixed an issue where sensors would occasionally be in the wrong mode and send garbage data

## [0.2.3] - 2026-02-11

### Added

- Derived `Copy` and `Clone` for the `Color` enum.

## [0.2.2] - 2026-01-27

### Fixed

- Fixed typo in `Motor` documentation.
- Fixed the documentation in `GyroController` documentation to include borrowing in the `new` function.

## [0.2.1] - 2026-01-24

### Fixed

- Fixed incorrect join and select usage in docs.

## [0.2.0] - 2026-01-24

### Fixed

- Fixed some incorrect examples in documentation

### Added

- Added the join and select macros. Select mimics the behavior of pybricks' racing multitask
  and join mimics the behavior of pybricks' non-racing multitask.
- Added multitasking examples to docs. 