# AeroStage

AeroStage saves AeroSpace window-to-workspace assignments and puts them back later.

It talks to the [AeroSpace](https://github.com/nikitabobko/AeroSpace) CLI. It does not replace AeroSpace.

## Requirements

- [AeroSpace](https://github.com/nikitabobko/AeroSpace) installed, with the `aerospace` binary on your `PATH`

## Install

Install AeroStage with Homebrew:

```sh
brew install jonahschueller/tap/aerostage
```

A Homebrew install makes `aerostage` available on the PATH AeroSpace uses, so you can call it directly from your AeroSpace config. For example:

```toml
[mode.main.binding]
alt-shift-s = 'exec-and-forget aerostage restore work'
```

You can build from source with `cargo install --path .`, but a custom install may not be on AeroSpace's PATH. In that case, see [AeroSpace's exec environment variables guide](https://nikitabobko.github.io/AeroSpace/guide#exec-env-vars) to configure its PATH.

## Commands

AeroStage saves named **stages**: snapshots of which windows belong on which AeroSpace workspaces. Capture one after arranging your windows, then restore it later. Restore moves windows that are already open; it does not launch applications.

```sh
aerostage capture work
aerostage restore work
aerostage list
aerostage show work
```

- `capture [stage]` saves the current workspace arrangement. Omit the stage name to use the default stage. Use `--stdout` to print the stage instead of saving it.
- `restore [stage]` restores a saved arrangement. Windows AeroStage cannot match are left where they are.
- `list` shows the saved stages. Add `--json` for JSON output.
- `show [stage]` prints a stage without restoring it.

## Capture

```sh
aerostage capture work
```

Saves the current assignment as `work.toml`. The stage's `name` is that filename, unless you pass `--name`.

Omit the stage name to write the default stage:

```sh
aerostage capture
```

Print the stage to stdout instead of writing a file:

```sh
aerostage capture --stdout
```

`--stdout` cannot be combined with a stage name. A stdout capture has no `name` unless you pass `--name`.

Capture only some workspaces:

```sh
aerostage capture work --workspaces 1,2,3
```

Capture also records each workspace's current layout. `--workspaces` takes a comma-separated list of AeroSpace workspace names. You can use `--default-workspace` to choose where leftover windows go when restoring:

```sh
aerostage capture work --default-workspace 1
```

## Restore

```sh
aerostage restore work
```

Loads `work.toml` and moves matching windows onto the workspaces recorded in the stage.

Omit the stage name to restore the default stage:

```sh
aerostage restore
```

Windows that cannot be matched are left where they are. AeroStage prints a line for each of those on stderr. Use `--from-workspaces` to limit which current workspaces AeroStage looks in:

```sh
aerostage restore work --from-workspaces 1,2
```

## Stage files

A stage is saved as a `.toml` file. You can edit it by hand; a captured file looks like this:

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

| Field               | Meaning                                                                                                                                                                    |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `description`       | Free-text note. Capture leaves this empty.                                                                                                                                 |
| `default_workspace` | If set, leftover live windows (ones the stage did not claim) are moved here. Windows that still look like they belong to an unmatched stage entry are left where they are. |
| `workspace.layout`  | One of `h_tiles`, `v_tiles`, `h_accordion`, or `v_accordion`. Restore flattens the workspace tree, then applies this layout.                                               |

You can drop `title`, `app`, or `bundle_id` when you edit a stage. Restore uses whatever is present to identify windows. If both `app` and `bundle_id` are set, a live window must match both.

Capture records workspace assignments and layouts, but not window focus. Restore matches using app identity and title, then uses workspace and app identity to resolve remaining windows. If a match is ambiguous, or a stage entry has no matching open window, AeroStage leaves that window in place.

## More options

You can also build from source with `cargo install --path .`.

## Configuration

AeroStage works without configuration. To change the default stage or stage directory, create `~/.aerostage/config.toml` or pass another file with `--config path`:

```toml
default_stage = "work.toml"
stage_directory = "/path/to/stages"
```

Without a custom configuration, the default stage is `default.toml` and stages are stored in `~/.aerostage/stages`. The stage directory is created when saving a stage.

When running with `cargo run`, stages are stored in the current working directory.
