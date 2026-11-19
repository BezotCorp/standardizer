# Standardizer

Standardizer checks or fixes the conformance of files in a Git repository by dispatching them to `rustfmt` or Biome.

## Installation

Download the archive for your platform from the GitHub Releases page, extract it, and place the `standardizer` executable in a directory listed in your `PATH`.

Standardizer expects the following tools to be available when they are required by the files in a repository:

- Git;
- Cargo and `rustfmt` for Rust projects;
- Biome for JavaScript, TypeScript, JSON, CSS, and GraphQL files.

Biome is resolved from `node_modules/.bin` in the target repository first, then from `PATH`. Install `rustfmt` with `rustup component add rustfmt` when using a rustup-managed toolchain.

## Usage

Run Standardizer from anywhere inside the Git repository to process:

```sh
standardizer check
standardizer fix
```

`check` reports files that do not conform without modifying them. `fix` applies supported corrections and stages only the files it changed. Both commands return a non-zero exit code when processing fails.

Standardizer processes regular files whose working-tree content still matches the Git index. It skips untracked, deleted, partially staged, and unstaged modified files, and refuses to run while the index contains unresolved conflicts.

## Routing

The distributed executable contains its routing configuration. Rust files are handled by `rustfmt`; JavaScript, TypeScript, JSON, CSS, and GraphQL files are handled by Biome. Target repositories do not need to provide a routing file.

## Building from source

Building requires a Rust toolchain compatible with edition 2024. The default development build reads the RON routing source directly:

```sh
cargo build --locked
```

To build the distributable executable, generate the binary routing data and enable only the embedded routing mode:

```sh
cargo run --locked -p generate-routing
cargo build --release --locked -p standardizer --no-default-features --features embedded-routing
```

The generated `data/standardizer/routing.bin` is ignored by Git. Regenerate it after changing `data/standardizer/routing.ron`.
