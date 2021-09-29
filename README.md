# toggl-cli

Unofficial CLI for [Toggl Track](https://toggl.com/track/) written in Rust, using the (undocumented) v9 API.

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

You can invoke the binary using the `toggl` command  now. Alternativly you can also run the command directly using `cargo run`

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
toggl 0.1.0
Toggl command line app.

USAGE:
    toggl [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    auth
    continue
    current
    help        Prints this message or the help of the given subcommand(s)
    list
    running
    start
    stop
```

You can also run the `help` command on a specific subcommand.

```shell
$ toggl help start
toggl-start 0.1.0

USAGE:
    toggl start [FLAGS] [OPTIONS] --description <description>

FLAGS:
    -b, --billable
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -d, --description <description>
    -p, --project <project>
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