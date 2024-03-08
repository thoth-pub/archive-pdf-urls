# Archive URL

This simple command-line tool archives URLs using the Wayback Machine.

## Installation

You can build and install the tool using Cargo:

```bash
cargo install archive-url
```

## Usage

The tool reads URLs from standard input, one URL per line, and archives them using the Wayback Machine.

Example usage:
```bash
echo "https://www.example.com" | archive-url
```
This will archive the URL "https://www.example.com".

