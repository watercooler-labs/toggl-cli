# Release runbook

Release preparation is local and reversible; publishing is explicit. Never use
production time entries for smoke tests. Do not publish while any gate below is
failing or unverified.

## Supported build matrix

| Artifact | Build / native smoke runner |
| --- | --- |
| `aarch64-apple-darwin` | macOS 15 ARM64 |
| `x86_64-apple-darwin` | macOS 15 Intel |
| `x86_64-unknown-linux-gnu` | Ubuntu 24.04 x86-64 |
| `x86_64-pc-windows-msvc` | Windows Server 2025 x86-64, static C runtime |

FreeBSD 15.0 has a source-build/test job, not a prebuilt release artifact.
FreeBSD users should use `TOGGL_API_TOKEN`; persistent keyring behavior is not
promised there. On Linux, the existing kernel-keyring backend may not survive a
reboot. Do not silently switch backends during a dependency upgrade.

Linux binaries dynamically link system libraries, including OpenSSL, and inherit the build runner's glibc requirements (Ubuntu 24.04 uses glibc 2.39).
Users of older distributions should build from source. macOS binaries are
smoke-tested on macOS 15; compatibility with older macOS versions needs a
separate check before being advertised. The Windows target changed from GNU to
MSVC in 0.6.0; update scripts that construct artifact names.

## Prepare a version

1. Start from a clean, up-to-date checkout of the intended release branch.
2. Use `./pkg/autodoc minor`, `./pkg/autodoc patch`, or an explicit version such
   as `./pkg/autodoc 0.6.0`. `--commit` can appear before or after the version.
   The helper uses pinned stable Rust, not nightly. It stops on command failure,
   restores version/docs files if preparation fails, and only stages the three
   release files when asked to commit.
3. Update `CHANGELOG.md`, noting compatibility changes, migration instructions,
   contributor PRs and any operational limitations. Review `SECURITY.md` against
   the latest audit rather than copying an old assessment.
4. Commit the version, lockfile, help snapshots and release notes with an
   emoji-prefixed imperative subject, e.g. `🚢 Prepare release 0.6.0`.

For docs-only changes, use `./pkg/autodoc --refresh`; CI checks drift with
`./pkg/autodoc --check`. Do not bump the version just to refresh help.

## Automated gates

Run from the committed checkout:

```sh
rustc --version                         # 1.100.0
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo audit --deny yanked
rustfmt --edition 2021 --check pkg/autodoc.rs
rustc --edition=2021 --test pkg/autodoc.rs -o target/autodoc-tests
target/autodoc-tests
python3 -m unittest discover -s pkg -p 'test_*.py'
shellcheck pkg/autodoc
./pkg/autodoc --check
cargo publish --dry-run --locked
```

Python tooling requires Python 3.11+. Workflow syntax is checked by
`go run github.com/rhysd/actionlint/cmd/actionlint@v1.7.12`.

The normal test suite uses mock keyrings and loopback HTTP servers. It must not
require secrets, a logged-in Toggl account, or an unlocked desktop keyring.
Informational dependency warnings are tracked in `SECURITY.md`; no vulnerability
advisories or yanked packages are allowed without a new explicit assessment.

## Create and inspect the draft

After pushing/merging the preparation commit (a separate maintainer action):

```sh
gh workflow run draft_release.yml --ref main
gh run list --workflow draft_release.yml
```

Confirm the workflow's `headSha` is the intended commit. The workflow runs CI,
builds all four native targets with `--locked`, and executes `--version`, help,
profile parsing and all completion generators from the **extracted archives**.
Only then does it create `v<version>` as a draft, with its target pinned to that
commit. It does not delete unrelated drafts or publish automatically. Reruns
can replace assets only on a draft for the same commit; a published release or
existing tag must never be overwritten.

A complete draft contains four ZIPs, four `<target>.sha256` files, `VERSION`, and
an installable `toggl-cli.json` Scoop manifest. Archives contain only the binary,
license and completion scripts. Download and verify before proceeding:

```sh
gh release download v0.6.0 --dir dist --clobber
python3 pkg/release.py verify --tag v0.6.0
```

The verifier requires all targets, matching versions and matching hashes. Inspect
generated GitHub notes and incorporate the curated changelog. Older `-next`
drafts are not release candidates for this process; remove obsolete drafts only
after confirming they are no longer needed.

## Manual smoke checks

Record the OS/version, artifact checksum, tested commit and result for each item.
A green cross-compile is not a native runtime check.

- [ ] Run the opt-in isolated keyring test in an unlocked native session on
  macOS, Windows and Linux:
  `cargo test --locked --test keyring -- --ignored --test-threads=1`.
  It uses a unique `togglcli-release-smoke` identity and a dummy value, never
  existing tokens, and cleans up its own credential.
- [ ] In a disposable OS account/VM, authenticate a **test Toggl account** using
  0.5.1's default credentials, then verify 0.6.0 reads them without reauthenticating.
- [ ] Authenticate two named test profiles, list profiles in text/JSON, check
  `--profile` before/after commands, and log out. Verify CLI > environment profile
  > config profile > environment token > default keyring precedence.
- [ ] In a disposable Toggl workspace, start/current/stop/continue a timer. Test
  duplicate task names across projects, `list task --project`, config overrides,
  editor-based edit, tags/projects, and cleanup. A misspelled project/task or
  cancelled picker must leave the existing timer running.
- [ ] Exercise `api` GET, a reversible write, empty DELETE responses, invalid
  authentication and rate-limit/error handling. Never record unredacted `/me`
  output: it includes the token.
- [ ] Load the generated completions in supported shells, not just generate them.
- [ ] Install the downloaded Windows manifest with Scoop, and test `toggl.exe`
  on a clean machine without a compiler toolchain installed.
- [ ] Confirm FreeBSD CI passes and smoke-test environment-token authentication
  if advertising FreeBSD runtime support in the release notes.

These gates are intentionally not marked complete by source-only tests. Real
account access and interactive/native environments require a maintainer.

## Publish the reviewed commit

Only after all gates and release notes are reviewed:

1. Verify the draft target and assets still match the tested commit, then publish
   the GitHub release in the UI or with
   `gh release edit v0.6.0 --draft=false --latest`. Do not move an existing tag.
2. Confirm **Publish Release Metadata** succeeds. It checks out the released tag,
   verifies the matching archives, and regenerates/uploads the Scoop manifest.
   For recovery, dispatch `release_publish.yml` with `tag=v0.6.0`.
3. From the same clean commit, run `cargo publish --locked` using a maintainer's
   crates.io credentials. This is deliberately manual; no registry secret or
   automatic crates.io publication is configured in this repository.
4. Verify `cargo install --locked toggl --version 0.6.0` in a clean environment.
   Confirm its version agrees with GitHub assets and `VERSION`.
5. Verify Scoop's generated URL/hash, and coordinate AUR `toggl-cli` and
   `toggl-cli-bin` version/hash updates with their maintainers. The workflow
   publishes a directly installable Scoop manifest; it does not update an
   external bucket or the AUR on anyone's behalf.

GitHub and crates.io publication are not atomic. If either fails, stop and record
which channel succeeded; retry the failed step without retagging or rebuilding
already published artifacts. Never claim package-manager availability before
verifying it. Do not yank a crate or delete a release as an automatic rollback.
