# sqript

Make a typescript of a terminal session in the style of the `script` command.

## Installation

```bash
$ cargo install sqript
```

## Usage

```bash
# Record a session
$ sqript <file name>

# Print help
$ sqript --help
```

## Comments

This is very much a toy implementation. I wrote this to play around with [pseudoterminals](https://man7.org/linux/man-pages/man7/pty.7.html) and [mio](https://docs.rs/mio).
