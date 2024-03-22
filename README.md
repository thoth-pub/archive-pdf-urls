# Archive PDF URLs

This command-line tool extracts URLs from a PDF file and archives them using the Wayback Machine.

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

