# ungit

[![License](https://img.shields.io/github/license/Censera/ungit.svg)](https://github.com/Censera/ungit/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/Censera/ungit/blob/main/Cargo.toml)
[![Last commit](https://img.shields.io/github/last-commit/Censera/ungit.svg)](https://github.com/Censera/ungit/commits/main)

A safety layer over Git for everyday workflows.

`ungit` wraps the Git operations that cause damage: losing commits,
force-pushing over someone else's work, and committing a secret. It calls Git.
It does not replace it.

## Building

Requires the `git` binary on `PATH`. `ungit` shells out to it, it does not
reimplement Git's object model.

```ts
cargo install ungit-cli
```

Building from source:

```ts
cargo install --path .
```

## Usage

```hs
ungit [OPTIONS] <COMMAND>

Commands:
  save    Stage changes and create a commit, refusing obvious mistakes
  sync    Fetch, rebase onto upstream, and push. Creates upstream if missing
  undo    Undo the last commit, keeping the working tree intact
  unsync  Revert the branch to its state before the last `sync`s rebase
  start   Fetch, update main, and create a new branch from it
  status  Show a human readable repository summary
  check   Detect repository problems
  repair  Repair problems found by `check`
  help    Print this message or the help of the given subcommand(s)
```

```ts
Options:
      --json     Emit machine readable JSON instead of formatted text, where supported
  -h, --help     Print help
  -V, --version  Print version
```

`undo --hard` discards the undone commit's changes instead of keeping them
in the working tree. Destructive, asks for confirmation. `unsync` also asks
for confirmation before rewriting the branch.

### Examples

```ts
ungit save "update readme"
ungit sync --remote origin
ungit undo --hard
ungit start feature/cleaning --from main
ungit check --allow ignored-files
ungit repair --yes
```

`repair` only auto fixes an in progress rebase (`merge-state`). Everything
else it rereports with the same fix hint `check` already gave you, you run the fix yourself.

## Contributing
 
Open an [issue](https://github.com/Censera/ungit/issues) or send a [PR](https://github.com/Censera/ungit/pulls).
