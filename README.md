# mDeno

A minimal JavaScript runtime for CLI tools. It's built on QuickJS and Rust.

## Install

```sh
cargo install --git https://github.com/ryuapp/mdeno mdeno
```

## How to use

```sh
# Run a JavaScript/TypeScript file
mdeno run hello.ts

# Compile a JavaScript/TypeScript file into a self-contained executable
mdeno compile hello.ts
```

## Supported Platforms

The tier system does not imply stability, but rather indicates the priority of addressing platform-specific bugs.

### Tier 1

- `x86_64-pc-windows-msvc`

### Tier 2

- `x86_64-unknown-linux-gnu`

### Tier 3

- `aarch64-unknown-linux-gnu`
- `aarch64-unknown-linux-musl`
- `x86_64-unknown-linux-musl`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `aarch64-pc-windows-msvc`

## Motivation

Starting with `deno compile`, we've made it easy to distribute JavaScript runtimes and scripts as single executable applications. Bun also has similar functionality, and Node.js can also be made into a single binary using external tools. However, the binary size for all of them is too large, making them unsuitable for simple CLI tools.

To solve this issue, we are developing a new JavaScript runtime using QuickJS, a lightweight JavaScript engine.
