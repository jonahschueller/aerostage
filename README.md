# AeroStage

AeroStage saves AeroSpace window-to-workspace assignments and puts them back later.

It talks to the [AeroSpace](https://github.com/nikitabobko/AeroSpace) CLI. It does not replace AeroSpace.

> [!WARNING]
> AeroStage is under active development. The CLI, stage format, and matching
> rules can still change, and there is no migration path yet. Treat this as a
> beta: useful, but not a stable contract. 

## Requirements

- [AeroSpace](https://github.com/nikitabobko/AeroSpace) installed, with the `aerospace` binary on your `PATH`
- Rust (to build from source)

## Install

```sh
cargo install --path .
```

That puts `aerostage` on your path. A release build stores stages in `~/.aerostage` (created on first run).

`cargo run` uses a debug build, which writes stages into the current working directory instead.

## Stages

A **stage** is a TOML snapshot of which windows belong on which workspace. Capture writes one from the live AeroSpace tree. You can also write or edit a stage by hand.

A stage does **not** launch apps, change layouts, or bind workspaces to monitors. Restore only moves windows that are already open.

Typical uses:

- Put windows back after a reboot
- Switch between named setups (work, coding, …)
- Keep a few stages for different monitor setups and restore the one you need

## Capture

```sh
aerostage capture work.toml
```

Writes the current assignment into `work.toml` in the stage directory.

Omit the filename to print TOML to stdout:

```sh
aerostage capture
```

Capture only some workspaces:

```sh
aerostage capture work.toml --workspaces 1,2,3
```

`--workspaces` is a comma-separated list of AeroSpace workspace names.

## Restore

```sh
aerostage restore work.toml
```

Loads that file from the stage directory and moves matching windows onto the workspaces recorded in the stage.

Windows that cannot be matched are left where they are. AeroStage prints a line for each of those on stderr.

## Stage files

Stages live next to each other as `.toml` files. A captured file looks like this:

```toml
name = "work.toml"

[[workspace]]
name = "1"

[[workspace.window]]
app = "Safari"
title = "Inbox"
bundle_id = "com.apple.Safari"

[[workspace]]
name = "2"

[[workspace.window]]
app = "Code"
title = "aerospace-arrangement"
bundle_id = "com.microsoft.VSCode"
```

Optional fields:

| Field | Meaning |
| --- | --- |
| `description` | Free-text note. Capture leaves this empty. |
| `default_workspace` | If set, leftover live windows (ones the stage did not claim) are moved here. |

You can drop `title` (or `app` / `bundle_id`) when you edit a stage. Restore uses whatever is present to identify windows.

## How restore matches windows

Window ids change every launch, so restore scores open windows against each entry in the stage. It tries, in order:

1. Same app, same title (case-insensitive; the stored title is also treated as a regex)
2. Same app, similar title (enough overlap to ignore small title changes)
3. Same app, already on the target workspace, and only one such window
4. Unique bundle id among remaining windows
5. Unique app name among remaining windows

A rule only wins if it points at exactly one window. Ambiguous cases fall through to the next rule.

## Notes

- Restore never starts applications. Open them first, then restore.
- Capture records assignments only, not tiling layout or focus.
- The filename you pass to `capture` / `restore` is joined with the stage directory. It is not a free-form path.
