# World Editor

![AI-assisted](https://img.shields.io/badge/AI--assisted-yes-8250df)

A browser-based world editor for the [fenbein3d engine](https://github.com/brstrutt/wasmfenbein3d). This is an early prototype: you can add and move wall lines, edit line coordinates and paintings, manage separate wall and painting texture lists, and import wall data from a `JsonWorld` JSON object. In the wall preview, scroll or use the zoom buttons to zoom, drag empty space to pan, and drag paintings to reposition them.
To see the latest live version go to [https://brstrutt.github.io/fenbein3d-world-editor/](https://brstrutt.github.io/fenbein3d-world-editor/)

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
