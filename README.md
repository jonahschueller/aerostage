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
aerostage watch work
aerostage list
aerostage show work
```

- `capture [stage]` saves the current workspace arrangement. Omit the stage name to use the default stage. Use `--stdout` to print the stage instead of saving it.
- `restore [stage]` restores a saved arrangement. Windows AeroStage cannot match are left where they are.
- `watch [stage]` restores a stage, then keeps it in sync with AeroSpace while it runs.
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

Workspace names in a stage and in `--from-workspaces` can be `@focused`; see [Focused workspace](#focused-workspace).

## Watch

```sh
aerostage watch work
```

Restores `work.toml` once, then keeps running and updates the stage file when its windows change. Omit the stage name to watch the default stage. Start it with AeroSpace:

```toml
after-startup-command = ['exec-and-forget aerostage watch']
```

Watch only manages the windows the stage describes. Each entry in the stage is either bound to a live window or waiting for one:

- When a bound window moves to another workspace, the stage follows it.
- When a bound window closes, its entry stays in the stage and waits. Quitting apps or shutting down the Mac therefore leaves the stage as it was.
- When a new window opens, it can claim a waiting entry: by title first, then by app. Watch moves it to the entry's workspace. If the window had focus, focus follows it.

Watch keeps your edits:

- A title is only updated if it was an exact copy of the window title when the window was bound. Titles you edited stay as written.
- An entry under `@focused` stays `@focused` until its window leaves the workspace `@focused` stood for.
- A workspace's `layout` is updated only if the stage already sets one and a bound window is on that workspace.

Limits:

- Only one watch can run per stage. While it runs, `capture` refuses to write that stage; `capture --stdout` still works.
- Watch never adds windows to the stage. To add new ones, stop watch, capture the stage again, and start watch again.
- Watch reads the stage file only when it starts. Edits made while it runs are overwritten by its next save, and saving drops comments from the file.
- There is no stop command. Quit it with `pkill -f "aerostage watch"`.

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

## Focused workspace

Use `@focused` wherever a stage or restore takes a workspace name to mean the workspace that is focused when you run `restore`. The name matches AeroSpace's `list-workspaces --focused`.

Bring an app to wherever you are:

```toml
[[workspace]]
name = "@focused"

[[workspace.window]]
app = "Microsoft Teams"
```

Move every leftover window to the workspace you are looking at:

```toml
default_workspace = "@focused"
```

Restore only windows that are on the focused workspace:

```sh
aerostage restore work --from-workspaces @focused
```

You can also capture with `--default-workspace @focused`; the stage keeps `@focused` rather than the workspace that was focused during capture.

Restore looks up the focused workspace once, before it moves any windows, and only if the stage or `--from-workspaces` uses `@focused`. If two workspace entries end up naming the same workspace (for example `@focused` and `"3"` while workspace 3 is focused), their windows are combined. If both entries set a `layout` and the layouts differ, restore stops with an error.

`@focused` is reserved: a workspace in AeroSpace that is literally named `@focused` cannot be referenced from a stage.

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

## Logs

AeroStage writes to `~/Library/Logs/aerostage/aerostage.log`, which Console.app also shows. Every command logs its errors. `capture`, `restore` and `watch` also log what they changed. This helps when AeroStage runs from AeroSpace, where nothing it prints is shown:

```
2026-11-26T10:12:03.123+01:00 INFO  [watch 4123] restored stage 'work': bound 5/7 entries
2026-11-26T10:15:20.877+01:00 INFO  [watch 4123] bound Safari "Docs", moving it to workspace 2
```

Each line names the command and its process id, so several runs can share the file. When the log grows past 5 MB it moves to `aerostage.log.1`. The three newest old logs are kept, and older ones are deleted.

Change the level or the file in the configuration:

```toml
log_level = "debug"   # off, error, warn, info (default), debug or trace
log_file = "/path/to/aerostage.log"
```

`AEROSTAGE_LOG=debug` overrides the level for one run.
