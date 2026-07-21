[![License](https://img.shields.io/github/license/Censera/ungit.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)

`ungit` is a safety layer over Git for everyday workflows. it wraps the Git operations most likely to cause damage (losing commits, force-pushing over someone else's work, committing a secret) and refuses or warns before they go through. It calls `git`; it doesn't reimplement Git's object model.

## Install

```ts
cargo install ungit-cli
```

Requires the `git` binary on `PATH`.

**From source:**

```ts
cargo install --path .
```

## Commands

```ts
ungit save <MESSAGE>    Stage all changes and commit, refusing obvious mistakes
ungit sync              Fetch, rebase onto upstream, and push (publishes branch if no upstream)
ungit undo              Undo the last commit, keeping the working tree intact
ungit unsync            Revert the branch to its state before the last sync's rebase
ungit start <BRANCH>    Fetch, update main, and create a new branch from it
ungit status            Show a human-readable repository summary
ungit check             Detect repository problems
ungit repair            Fix problems found by check
```

```ts
Options:
      --json    Emit machine-readable JSON where supported
  -h, --help
  -V, --version
```

## What `save` checks

Before staging, `save` scans every changed path for:

- Filenames that look like they contain secrets (private keys, credential files, `.env`)
- Files that are unusually large for source

Pass `--force` to commit anyway if you know what you're doing.

## What `check` detects

- Detached HEAD
- Branch diverged from upstream
- Duplicate patch already applied upstream
- Tracked files that should be ignored
- Interrupted merge or rebase state
- Missing upstream tracking reference

`repair` auto-fixes an in-progress rebase (`merge-state`). Everything else it re-reports with the same fix hint `check` already gave you; run the fix yourself.

## Examples

```ts
ungit save "update readme"
ungit save --force "add .env.example"
ungit sync
ungit sync --remote upstream
ungit undo
ungit undo --hard
ungit start feature/login --from main
ungit check
ungit check --allow ignored-files
ungit repair --yes
```

`undo --hard` discards the undone commit's changes rather than staging them. Destructive; asks for confirmation. `unsync` also asks before rewriting the branch.

## Contributing

Open an [issue](https://github.com/Censera/ungit/issues) or send a [PR](https://github.com/Censera/ungit/pulls).

## License

[Apache 2.0](LICENSE)
