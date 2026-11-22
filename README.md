# Standardizer

Standardizer checks or fixes the conformance of files in a Git repository by
dispatching them to `rustfmt` or Biome.

## Installation

```sh
cargo install standardizer --locked
```

Standardizer requires Git and the formatting tools needed by the files being
processed. Rust files use `rustfmt`; JavaScript, TypeScript, JSON, CSS and
GraphQL files use Biome.

## Usage

```sh
standardizer check
standardizer fix
```

`check` reports non-conforming files without modifying them.

`fix` applies supported corrections and stages only the files it changed.

## Routing

The default executable embeds `data/routing.bin`.

The source representation is stored in `data/routing.ron`.

The routing generator is maintained separately in the
`BezotCorp/standardizer-generate-routing` repository.

## Building

```sh
cargo build --locked
```

To use the RON routing source directly during development:

```sh
cargo build --locked --no-default-features --features routing-source
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
