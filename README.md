# Archive PDF URLs

This command-line tool extracts URLs from a PDF file and archives them using the Wayback Machine.

[![Build status](https://github.com/thoth-pub/archive-pdf-urls/workflows/test-and-check/badge.svg)](https://github.com/thoth-pub/archive-pdf-urls/actions)
[![Crates.io](https://img.shields.io/crates/v/archive-pdf-urls.svg)](https://crates.io/crates/archive-pdf-urls)

## Installation

You can build and install the tool using Cargo:

```bash
cargo install archive-pdf-urls
```

## Usage

The tool reads URLs from standard input, one URL per line, and archives them using the Wayback Machine.

Example usage:
```bash
archive-pdf-urls file.pdf --exclude https://some.pattern/\*
```

### Docker usage

```bash
docker run --rm -v ./file.pdf:/file.pdf ghcr.io/thoth-pub/archive-pdf-urls file.pdf
```
