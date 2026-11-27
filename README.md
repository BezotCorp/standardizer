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

```sh
standardizer tools            # open the graphical tool manager
standardizer tools status     # show available tool installations
standardizer tools install biome
standardizer tools uninstall biome
```

`standardizer --help` (or `<command> --help`) prints usage, and
`standardizer --version` prints the installed version.

## Data

Standardizer embeds the generated binary data from
`data/routing.bin` and `data/tools.bin`.

The source representations are stored in `data/routing.ron`
and `data/tools.ron`.

Only the RON sources are tracked in Git. Binary data is generated
locally and during CI/CD.

The data generator is maintained separately in the
`BezotCorp/standardizer-generate-data` repository.
Its required version is defined in `scripts/generate-data.sh`.

## Getting started

After cloning the repository, initialize the development environment
before building or committing changes:

```sh
./scripts/bootstrap.sh
```

The bootstrap script checks development prerequisites, installs
Standardizer if necessary, installs the required data generator,
generates the binary data and activates the Git hooks.

The script can be run again safely.

The pre-commit hook regenerates the binary data before running
`standardizer fix`.

To regenerate the binary data manually during development:

```sh
./scripts/generate-data.sh
```

CI/CD uses the same generation script before compilation and
crate publication.

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
