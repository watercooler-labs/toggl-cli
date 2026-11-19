# toggl-cli

Unofficial CLI for [Toggl Track](https://toggl.com/track/) written in Rust, using the [v9 API](https://developers.track.toggl.com/docs/).

## Usage

Building the binary.

```shell
cargo build # or cargo build --release
```

Installing the binary.

### From [crates.io](https://crates.io/crates/toggl)

```shell
cargo install toggl
```

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
cargo install --path .
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

### Commands

Run the `help` command to see a list of available commands.

```shell
$ toggl help
toggl 0.5.1
Toggl command line app.

USAGE:
    toggl [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --fzf        Use fzf instead of the default picker
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -C <directory>         Change directory before running the command
        --proxy <proxy>    Use custom proxy

SUBCOMMANDS:
    api               Issue a raw GET request against the Toggl v9 API and
                      print the JSON response
    auth              Authenticate with the Toggl API. Find your API token at https://track.toggl.com/profile#api-
                      token
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
toggl-start 0.5.1
Start a new time entry, call with no arguments to start in interactive mode

USAGE:
    toggl start [FLAGS] [OPTIONS] [--] [description]

FLAGS:
    -b, --billable
    -h, --help           Prints help information
    -i, --interactive
    -V, --version        Prints version information

OPTIONS:
    -p, --project <project>    Exact name of the project you want the time entry to be associated with
    -t, --tags <tags>...       Space separated list of tags to associate with the time entry, e.g. 'tag1 tag2 tag3'
        --task <task>          Exact name of the task you want the time entry to be associated with

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
cargo test
```

## Linting

Common lint tools

```shell
cargo fmt # Formatting the code to a unified style.

cargo clippy --fix # To automatically fix common mistakes.
```

The CI will also run the lint commands for all pull-requests.
See [pull_request.yml](.github/workflows/pull_request.yml) for more details.

## Releasing

To create a new release, first bump the package version.
We have a handy script at [pkg/autodoc.rs](pkg/autodoc.rs).
Running it with no arguments, bumps the current patch version, in `Cargo.toml`
and `Cargo.lock` files.
It also updates the help documentation in this README to match the current
version of the command.
Commit and push the updated changes and a new release will show up under the
[releases](https://github.com/watercooler-labs/toggl-cli/releases) page.

---

Built by the [Watercooler Studio](https://watercooler.studio/)
