json2type
===

A CLI program to covert a JSON object to a type. Currently only Go structs are supported.

# Installation
```shell
cargo install --git https://github.com/triarius/json2type
```

# Usage
```
Usage: json2type [OPTIONS] --name <NAME>

Options:
  -n, --name <NAME>      Name for the struct
  -i, --input <INPUT>    File to read JSON, default: stdin
  -o, --output <OUTPUT>  File to output Go, default: stdout
  -h, --help             Print help
  -V, --version          Print version
```

# To-do
## Languages
- [x] Go
- [] Rust
- [] Typescript

## Features
- [x] Order preserving
- [] Format output
- [] Non-objects as type aliases
