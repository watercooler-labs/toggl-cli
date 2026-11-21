# toggl-cli

Unofficial CLI for [Toggl Track](https://toggl.com/track/) written in Rust, using the [v9 API](https://developers.track.toggl.com/docs/).

## Usage

Building from source requires **Rust 1.100.0 or newer**. The repository pins
1.100.0 via `rust-toolchain.toml` so rustup selects the tested compiler.

```shell
cargo build --locked # or cargo build --locked --release
```

On Debian/Ubuntu, install `build-essential pkg-config libssl-dev`
first. FreeBSD source builds require `pkgconf openssl`; use `TOGGL_API_TOKEN`
there rather than relying on a persistent native keyring.

Installing the binary.

### From [crates.io](https://crates.io/crates/toggl)

```shell
cargo install --locked toggl
```

### Prebuilt binaries and Windows Scoop

Download the archive for your platform from [GitHub releases](https://github.com/watercooler-labs/toggl-cli/releases).
Each archive includes the binary, license and completion scripts, with a
matching `<target>.sha256` checksum file. Verify the checksum before installing.

Starting with 0.6.0, Windows archives use the native **MSVC** target rather than
GNU. A generated Scoop manifest is attached to each release:

```powershell
scoop install https://github.com/watercooler-labs/toggl-cli/releases/latest/download/toggl-cli.json
```

See the [release runbook](docs/releasing.md) for build-platform and runtime checks.

### From Arch Linux aur
[![toggl-cli](https://img.shields.io/aur/version/toggl-cli?label=toggl-cli)](https://aur.archlinux.org/packages/toggl-cli/)
[![toggl-cli-bin](https://img.shields.io/aur/version/toggl-cli-bin?label=toggl-cli-bin)](https://aur.archlinux.org/packages/toggl-cli-bin/)

toggl-cli is available on the [AUR](https://wiki.archlinux.org/index.php/Arch_User_Repository):
- [toggl-cli](https://aur.archlinux.org/packages/toggl-cli/) (release package)
- [toggl-cli-bin](https://aur.archlinux.org/packages/toggl-cli-bin/) (binary release package)

You can install it using your [AUR helper](https://wiki.archlinux.org/index.php/AUR_helpers) of choice.

Example:
```shell
$ yay -Sy toggl-cli
```

### From source

```shell
cargo install --locked --path .
```

> This places the release optimized binary at `~/.cargo/bin/toggl`. Make sure to add `~/.cargo/bin` to your `$PATH` so that you can run the binary from any directory.

You can invoke the binary using the `toggl` command now. Alternatively you can also run the command directly using `cargo run`

```shell
cargo run [command]


# To list the last 3 time-entries
cargo run list -n 3
```

The first command you need to run is `auth` to set up your [Toggl API token](https://support.toggl.com/en/articles/3116844-where-is-my-api-token-located).

```shell
cargo run auth [API_TOKEN] # or toggl auth [API_TOKEN]
```

The API token is stored securely in your Operating System's keychain using the [keyring](https://crates.io/crates/keyring) crate.

> **Note**: On some linux environments the `keyring` store is not persistent
> across reboots. We recommend exporting the api token as `TOGGL_API_TOKEN`
> in your shell profile. The CLI will use this environment variable if it is
> set. You don't need to run the `auth` command if you have the environment
> variable set.

### Profiles

If you have multiple Toggl accounts (e.g. work and personal), you can
store separate tokens under named profiles.

```shell
toggl auth --profile work
toggl auth --profile personal
```

Select a profile in three ways (listed in priority order):

1. **CLI flag**: applies to a single command:
   ```shell
   toggl --profile work start
   toggl --profile personal list
   ```

2. **Environment variable**: applies to all commands in the session:
   ```shell
   export TOGGL_PROFILE=work
   toggl start
   ```

3. **TOML config**: applies per-directory/branch via your auto-tracking
   config:
   ```toml
   ['*']
   profile = "work"
   project = "{{current_dir}}"
   ```

If no profile is specified, the CLI uses the `default` profile, which is
fully backwards compatible with existing setups.

**Priority order:**
1. Explicit `--profile` flag → always uses keyring (ignores `TOGGL_API_TOKEN`)
2. `TOGGL_PROFILE` environment variable → uses keyring for that profile
3. `profile` field in TOML config → uses keyring for that profile
4. `TOGGL_API_TOKEN` environment variable → uses the token directly (no keyring)
5. Keyring `default` profile → fallback

`TOGGL_API_TOKEN` is used only when **none** of `--profile`, `TOGGL_PROFILE`,
or a TOML `profile` selects a stored credential. An explicit CLI or environment
profile skips config-based profile lookup. The `--profile` flag works before or
after a subcommand, including `auth` and `logout`.

For example, with no environment or config profile selected:

```shell
export TOGGL_API_TOKEN=xxx  # Used by default
toggl start                  # Uses TOGGL_API_TOKEN
toggl --profile work start   # Uses keyring "work" profile (ignores env var)
```

To clear a specific profile's stored token:

```shell
toggl logout --profile work
```

To list all stored profiles and their associated email addresses:

```shell
$ toggl list profile
  default    user@example.com
* work       user@company.com  (active)
  personal   user@personal.com
```

The `*` marker indicates the currently active profile based on your
`--profile` flag, `TOGGL_PROFILE` environment variable, or TOML config.
The `default` profile is always listed, even if it was never registered
with `toggl auth --profile`.

For scripting, pass `--json` to get machine-readable output:

```shell
$ toggl list profile --json
[
  {
    "name": "default",
    "email": "user@example.com",
    "active": false
  },
  {
    "name": "work",
    "email": "user@company.com",
    "active": true
  }
]
```

Note: `toggl list profile` ignores the date filters (`--since`,
`--until`) and the `-n` limit, which apply only to the other entities.

### Shell completions

The `completions` command prints a completion script for `bash`, `zsh`,
`fish`, `powershell` or `elvish` to stdout.

```shell
# bash
toggl completions --shell bash > ~/.local/share/bash-completion/completions/toggl

# zsh, any directory in your $fpath
toggl completions --shell zsh > ~/.zfunc/_toggl

# fish
toggl completions --shell fish > ~/.config/fish/completions/toggl.fish
```

### Commands

Run the `help` command to see a list of available commands.

```shell
$ toggl help
toggl 0.6.0
Toggl command line app.

USAGE:
    toggl [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --fzf        Use fzf instead of the default picker
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -C <directory>             Change directory before running the command
        --profile <profile>    Profile name for API token (e.g. 'work', 'personal')
        --proxy <proxy>        Use custom proxy

SUBCOMMANDS:
    api               Issue a raw request against the Toggl API and print the JSON response
    auth              Authenticate with the Toggl API. Find your API token at https://track.toggl.com/profile#api-
                      token
    completions       Create tab-completion scripts
    config            Manage auto-tracking configuration
    continue
    create-project    Create a new project in your workspace
    create-tag        Create a new tag in your workspace
    current
    delete            Delete a time entry by ID
    delete-project    Delete a project from your workspace by name
    delete-tag        Delete a tag from your workspace by name
    edit              Edit a time entry, opens it in $EDITOR when no edit flags are given
    help              Prints this message or the help of the given subcommand(s)
    list
    logout            Clear stored credentials
    rename-project    Rename a project in your workspace
    rename-tag        Rename a tag in your workspace
    running
    start             Start a new time entry, call with no arguments to start in interactive mode
    stop

```

You can also run the `help` command on a specific subcommand.

```shell
$ toggl help start
toggl-start 0.6.0
Start a new time entry, call with no arguments to start in interactive mode

USAGE:
    toggl start [FLAGS] [OPTIONS] [--] [description]

FLAGS:
    -b, --billable
    -h, --help           Prints help information
    -i, --interactive
    -V, --version        Prints version information

OPTIONS:
        --profile <profile>    Profile name for API token (e.g. 'work', 'personal')
    -p, --project <project>    Exact name of the project you want the time entry to be associated with
    -t, --tags <tags>...       Space separated list of tags to associate with the time entry, e.g. 'tag1 tag2 tag3'
        --task <task>          Exact name of the task you want the time entry to be associated with, requires --project

ARGS:
    <description>    Description of the time entry

```

### Raw API access

For debugging and exploration, the `api` command issues a raw request
against the Toggl API using your stored credentials and prints the JSON
response. The response is printed untouched, so you can inspect fields that
the CLI's model layer does not surface (for example `duronly` or `at` on a
time entry), or check the live schema before adding a new endpoint to the
client.

```shell
# The currently running time entry, exactly as the API returns it
$ toggl api /me/time_entries/current

# Compose with jq for ad-hoc queries
$ toggl api /me/projects | jq '.[].name'

# Error responses include the HTTP status and the response body
$ toggl api /nonexistent/endpoint
HTTP 404 Not Found: 404 page not found
```

The `api` command follows Unix conventions: exit code 0 on success and 1
on failure, JSON data on stdout, and errors on stderr. Any non-2xx HTTP
status is a failure, so stdout never mixes success data with error
bodies.

By default the request is a GET against the
[v9 API](https://engineering.toggl.com/docs/api/time_entries). Pass an
absolute URL to reach other APIs on `api.track.toggl.com`, such as the
[reports API](https://engineering.toggl.com/docs/reports/):

```shell
# A detailed report via the v3 reports API
toggl api \
  https://api.track.toggl.com/reports/api/v3/workspace/{wid}/search/time_entries \
  -X POST -f start_date=2026-11-19 -f end_date=2026-11-20
```

Write requests are supported with `-X`, `-i` (raw JSON body from a file,
or `-` for stdin) and two repeatable `key=value` flags:

- `-f` / `--raw-field` sends the value as a string, always
- `-F` / `--field` parses the value as a typed parameter: booleans,
  integers and floats are sent as such, anything else as a string. Use
  `@<path>` or `@-` to read the value from a file or stdin

Empty responses (e.g. HTTP 204 from a `DELETE`) print nothing:

```shell
# Create a tag
$ toggl api /workspaces/{wid}/tags -X POST -f name=inbox

# Numeric and boolean fields
$ toggl api /workspaces/{wid}/projects -X POST \
  -F name="new project" -F billable=true -F rate=5500

# Send a body from stdin
$ echo '{"start_date": "2026-11-19"}' \
  | toggl api \
    https://api.track.toggl.com/reports/api/v3/workspace/{wid}/search/time_entries \
    -X POST -i -
```

Replace `{wid}` with your workspace ID (see `toggl api /me/workspaces`).

> **Note**: because the response is printed untouched, it can contain
> secrets. For example `toggl api /me` includes your API token. Pipe
> carefully, e.g. `toggl api /me | jq 'del(.api_token)'` when sharing
> output.

Some useful paths:

```shell
toggl api /me                         # your user profile
toggl api /me/time_entries            # time entries
toggl api /me/time_entries/current    # the running entry
toggl api /me/projects                # your projects
toggl api /me/workspaces              # your workspaces
```

## Testing

To run the unit-tests

```shell
cargo test --locked --all-targets
```

Tests use mock credentials and local HTTP servers, never a live Toggl account.
The isolated native-keyring round-trip is opt-in; see the
[release checklist](docs/releasing.md#manual-smoke-checks).

## Linting

Common lint tools

```shell
cargo fmt # Formatting the code to a unified style.

cargo clippy --fix # To automatically fix common mistakes.
```

The CI will also run the lint commands for all pull-requests.
See [pull_request.yml](.github/workflows/pull_request.yml) for more details.

## Releasing

Follow [docs/releasing.md](docs/releasing.md) for the complete checklist,
manual smoke tests, publishing steps and package-manager verification.

From a clean checkout, `./pkg/autodoc minor` prepares a minor release (no
argument means a patch release). It updates `Cargo.toml`, `Cargo.lock` and this
README using the pinned stable compiler. Use `--commit` to commit those files,
`--refresh` to regenerate help without bumping, or `--check` to check for drift.

Releases are **not** published automatically on push. Dispatch **Draft Release**
for the intended commit after merging the version and release notes. It gates
the draft on CI and native artifact smoke checks. Publishing to GitHub and
crates.io remains an explicit maintainer action.

See [CHANGELOG.md](CHANGELOG.md) for user-facing changes and
[SECURITY.md](SECURITY.md) for the dependency audit policy.

---

Built by the [Watercooler Studio](https://watercooler.studio/)
