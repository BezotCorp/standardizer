# Standardizer

Standardizer checks or fixes the conformance of files in a Git repository by
dispatching them to `rustfmt` or Biome.

## Installation

```sh
cargo install standardizer --locked
```

Standardizer requires Git and the formatting tools needed by the files being
processed. Rust files use `rustfmt`; JavaScript, TypeScript, JSON, JSONC, CSS,
GraphQL, Markdown and YAML files use Biome.

## Usage

```sh
standardizer check
standardizer fix
```

`check` reports non-conforming files without modifying files or the Git index.

`fix` applies supported corrections, stages the resulting changes and handles
file deletions according to the repository configuration.

Both commands process tracked files whose working tree content matches the
Git index.

### File protection

Standardizer preserves:

- Untracked files.
- Tracked files with unstaged modifications.
- Partially staged files.
- Files whose content, type, executable mode or index entry changes
  unexpectedly during processing, when those changes can be detected.

Files without unstaged modifications remain eligible for processing,
including files that have not been explicitly staged.

Standardizer verifies file and index consistency before formatting and
before staging corrections. These checks reduce the risk of including
unrelated changes but do not provide atomic protection against concurrent
modifications.

### Automatic deletion staging

By default, `fix` automatically stages unstaged deletions when they are the
only remaining unstaged changes in the repository.

If any unstaged modification or non-ignored untracked file remains,
Standardizer does not automatically stage any deletion.

This policy also applies to deletions occurring during execution.

| Remaining unstaged changes    | Automatic deletion staging |
| ----------------------------- | -------------------------- |
| Deletions only                | Enabled                    |
| Deletions and modifications   | Disabled                   |
| Deletions and untracked files | Disabled                   |
| No deletions                  | Nothing to stage           |

Previously staged deletions are preserved.

Automatically staged deletions are reported individually.

### Tool management

```sh
standardizer tools            # open the graphical tool manager
standardizer tools status     # show available tool installations
standardizer tools install biome
standardizer tools uninstall biome
```

`standardizer --help` (or `<command> --help`) prints usage, and
`standardizer --version` prints the installed version.

## Configuration

Standardizer supports two optional JSON configuration files located at the
root of the Git repository:

| File                      | Purpose                          |
| ------------------------- | -------------------------------- |
| `.standardizer/team.json` | Shared repository configuration  |
| `.standardizer/dev.json`  | Personal developer configuration |

Configuration properties are resolved individually in the following order:

1. Team configuration.
2. Developer configuration.
3. Default values.

Neither configuration file is required.

Invalid JSON, unknown properties and unsupported values cause the operation
to fail before applying corrections.

### Deletion policy

Automatic deletion staging is enabled by default.

To disable it, add the following property to the team or developer
configuration:

```json
{
  "deletion": "ignore"
}
```

When this property is set, Standardizer does not automatically stage
deletions. Previously staged deletions remain unaffected.

Omit the property to use the default behavior when no higher-priority
configuration defines it.

The only supported explicit value is `"ignore"`.

### Developer configuration protection

The developer configuration is intended to remain local to each developer
and must not be committed.

When running `fix`, Standardizer:

- Adds `/.standardizer/dev.json` to the repository's local Git exclusions
  in `.git/info/exclude`.
- Preserves existing exclusion rules.
- Removes the developer configuration from the Git index if it is tracked,
  without deleting the local file.

Standardizer does not modify `.gitignore` or rewrite Git history.

The team configuration can be tracked and shared normally.

Running `check` does not modify configuration files, Git exclusions or
the Git index.

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

## Testing

Run the test suite:

```sh
cargo test --locked
```

Run Clippy with all targets and features:

```sh
cargo clippy --locked --all-targets --all-features -- -D warnings
```

The test suite covers configuration resolution, developer configuration
protection, Git index preservation, concurrent file modifications and
automatic deletion staging.

Integration tests use independent temporary Git repositories.

## Generated data and package integrity

The RON files are the version-controlled source of truth. Binary data files
are generated from these sources using the pinned version of
`standardizer-generate-data`.

Generated binary files are intentionally excluded from Git to prevent binary
merge conflicts and inconsistencies between source data and generated data.

The following generated artifacts are included in the published Cargo package:

- `data/routing.bin`
- `data/tools.bin`

Cargo does not distinguish intentionally generated package files from
unexpected working tree changes. Consequently, package verification and
publication use `--allow-dirty` only after passing an explicit integrity check.

The shared `scripts/check-package-integrity.sh` script verifies that:

- The Git index and working tree contain no unexpected changes.
- Both generated artifacts exist, are non-empty regular files, are ignored
  by Git, and are not tracked.
- Both generated artifacts are included in the Cargo package.
- Every other packaged source file is tracked by Git.
- Cargo-generated package metadata is not present in the source tree.

Any unexpected change or unauthorized package file aborts publication.

The same integrity validation is used during CI package verification and
actual crate publication. Cargo may mark the published package as dirty
because it contains the intentionally generated binary artifacts.

Consumers receive the generated binary data directly in the published crate
and do not need to install the data generator.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
