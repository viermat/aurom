# aurom

`aurom` is a command-line tool designed to automate Chrome browser interactions, specifically tasks that typically require user input or manual completion of challenges.

# Getting started

### Building from source

To build `aurom` from source, you need Rust 1.80.0 or later. Clone the repository and use `cargo` to build and install it:

```bash
cargo install --path .
```

To verify a successful installation, simply run:

```bash
aurom -h
```

# How to use

## Basic usage

`aurom` requires either the `-n` flag or the `-c` flag with a WebSocket URL provided when starting a Chrome browser and specifying the ``--remote-debugging-port`` flag:

```bash
$ aurom -n
```
or
```bash
$ aurom -c ws://127.0.0.1:9222/devtools/browser/b5fb02d9-b2a9-4a3f-83db-c63e7c68c300
```

> [!WARNING]  
> You can not use -n and -c at the same time.

You can specify the ``-u`` flag to set the URL of the target tab:

```bash
$ aurom -n -u https://example.com/
```

## Advanced usage

For a more advanced usage of `aurom`, please refer to `aurom --help`.