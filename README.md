# ungit

`ungit` is a small workflow layer over Git for everyday work. It deliberately hides Git's staging, upstream, rebase, and branch machinery behind a few safe actions.

The model is:

```text
start → work → save → sync
```

You work normally. When the work is worth keeping, save it. When it is ready to be shared, sync it. `ungit` handles the Git machinery underneath.

## Install

```sh
cargo install ungit-cli
```

Requires the `git` binary on `PATH`.

From source:

```sh
cargo install --path .
```

## Commands

```text
ungit start <NAME>      Start a new piece of work
ungit save <MESSAGE>    Save all current changes
ungit sync              Update and publish current work
ungit undo              Undo the last save
ungit status            Show the current state
```

That is the normal workflow. There is no staging step, no upstream setup, no manual rebase, and no recovery command users need to learn.

## Example

```sh
ungit start login

# work

ungit save "implement login"

# work

ungit save "fix validation"
ungit sync
```

`save` checks changed paths for obvious secrets and unusually large files before committing. Use `--force` only when you intentionally want to bypass those checks.

`undo` is an escape hatch for the last local save. `undo --hard` permanently discards its changes and requires care.

`status` is informational. It is not required to operate the workflow.

## Design

Git remains the storage and transport mechanism. `ungit` does not attempt to replace Git's object model or expose all of Git's concepts. Its job is to make the common lifecycle small, predictable, and difficult to break accidentally.

When `sync` needs to reconcile local work with remote work, it does that internally. If reconciliation fails, it aborts the operation instead of leaving the user inside an unfinished Git operation.

## License

Apache 2.0
