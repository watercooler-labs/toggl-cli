# toggl-cli

Unofficial CLI for [Toggl Track](https://toggl.com/track/) written in Rust, using the [v9 API](https://developers.track.toggl.com/docs/).

## Usage

Building the binary.

```shell
cargo build # or cargo build --release
```

Installing the binary.

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

### Commands

Run the `help` command to see a list of available commands.

```shell
$ toggl help
toggl 0.3.0
Toggl command line app.

USAGE:
    toggl [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --fzf        Use fzf instead of the default picker
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --proxy <proxy>    Use custom proxy

SUBCOMMANDS:
    auth
    config      Manage auto-tracking configuration
    continue
    current
    help        Prints this message or the help of the given subcommand(s)
    list
    running
    start       Start a new time entry. Call with no arguments to start in interactive mode.
    stop
```

You can also run the `help` command on a specific subcommand.

```shell
$ toggl help start
toggl-start 0.1.0
Start a new time entry. Call with no arguments to start in interactive mode.

USAGE:
    toggl start [FLAGS] [OPTIONS] [description]

FLAGS:
    -b, --billable
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -p, --project <project>

ARGS:
    <description>
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

---

Built by the [Watercooler Studio](https://watercooler.studio/)
