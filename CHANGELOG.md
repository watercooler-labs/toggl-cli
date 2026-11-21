# Changelog

## 0.6.0 — 2026-11-21

### Added

- Named API-token profiles for multiple accounts, with CLI, environment and
  per-directory/branch configuration selection; `list profile` supports JSON
  ([#101](https://github.com/watercooler-labs/toggl-cli/pull/101)).
- Raw `toggl api` access with HTTP methods, JSON files/stdin and typed fields
  ([#102](https://github.com/watercooler-labs/toggl-cli/pull/102)). Treat raw output
  as sensitive: `/me` includes the API token.
- Shell-completion generation for Bash, Zsh, Fish, PowerShell and Elvish
  ([#100](https://github.com/watercooler-labs/toggl-cli/pull/100)).
- `list task`, optional project filtering, and project-scoped task selection
  ([#98](https://github.com/watercooler-labs/toggl-cli/pull/98)).
- Source-build support for FreeBSD and other XDG Unix platforms
  ([#99](https://github.com/watercooler-labs/toggl-cli/pull/99)), with a FreeBSD
  CI job. Use an environment token where no persistent keyring is available.

### Fixed

- `--profile` now works before or after subcommands, including `auth` and `logout`.
  Existing default credentials retain the same keyring identity.
- Explicit CLI/environment profiles no longer parse unrelated config templates.
  Completion generation and config recovery work with malformed TOML; parse
  errors return a normal failure instead of panicking or polluting stdout.
- HTTP errors no longer masquerade as successful empty lists. Network/status
  failures propagate to stderr and a nonzero exit code; all requests have a
  timeout. Missing project references in task data no longer panic.
- Invalid project/task selections, config failures and cancelled interactive
  selections do not stop the running timer. Continue selects stopped entries
  without assuming the running entry is first or requiring a post-create refetch.
- Release preparation respects minor/explicit versions, checks subprocess
  failures and restores files on failure. Drafts are gated on complete builds
  and artifact smoke tests instead of being replaced on every push.

### Compatibility and packaging

- Source builds now require **Rust 1.100.0**; the repository pins that compiler.
- `start --task` requires `--project`; tasks are resolved within that project.
  A CLI project override does not retain a task from a different configured project.
- Scripts must now handle nonzero exit codes for network/API failures rather
  than interpreting an outage as an empty dataset.
- Credential precedence is CLI profile > `TOGGL_PROFILE` > config profile >
  `TOGGL_API_TOKEN` > default keyring. Named profiles override an environment token.
- Windows archives use `x86_64-pc-windows-msvc` instead of the GNU target and
  statically link the C runtime. A versioned Scoop manifest is included.
- Archives include completion scripts and the license, with per-target checksum
  files. See [the runbook](docs/releasing.md) for platform runtime requirements.
- Dependencies and the lockfile were refreshed. Known informational maintenance
  warnings and the atty Windows allocator assessment are documented in
  [SECURITY.md](SECURITY.md).

### Release validation

Automated checks, manual smoke checks and package-manager verification are
documented in [docs/releasing.md](docs/releasing.md).
