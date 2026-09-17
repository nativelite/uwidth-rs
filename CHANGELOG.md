# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-09-12

### Added
- **`char_width` and `str_width`**: the terminal column width of a `char`
  (0, 1 or 2) and of a `&str`. Zero for combining marks, default-ignorable
  format characters, conjoining Hangul jamo and C0/C1 controls; two for East
  Asian Wide and Fullwidth characters and `Emoji_Presentation`; one otherwise,
  including East Asian Ambiguous.
- Tables generated from the Unicode 16.0.0 Character Database by
  `tools/gen_tables.py` and vendored as range slices; lookup is a binary search.
  `UNICODE_VERSION` names the data version.
- `#![no_std]`, no `unsafe`, zero third-party dependencies.

[Unreleased]: https://github.com/nativelite/uwidth-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/nativelite/uwidth-rs/releases/tag/v0.1.0
