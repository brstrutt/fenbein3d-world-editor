# World Editor

![AI-assisted](https://img.shields.io/badge/AI--assisted-yes-8250df)

A browser-based world editor for the [fenbein3d engine](https://github.com/brstrutt/wasmfenbein3d). This is an early prototype: you can add and move wall lines, pan and zoom the scene, edit line coordinates, and import wall data from a `JsonWorld` JSON object.

The AI-assisted badge is a self-reported Shields.io badge; it is not an official GitHub verification.

## Getting started

The easiest way to get a working development environment is to open this repository in VS Code and reopen it in the dev container. It installs the WebAssembly target and Trunk for you.


## Development commands

Start the development server from the repository root:

```sh
trunk serve
```

Run the tests:

```sh
cargo test
```

Format the Rust code:

```sh
cargo fmt
```
