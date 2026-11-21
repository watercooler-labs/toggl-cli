# Security and dependency maintenance

Please report suspected credential exposure privately through the repository's
GitHub security advisory interface, not in a public issue containing tokens.
Never attach unredacted `toggl api /me` output: it includes the API token.

## Release checks

Use the pinned Rust toolchain and committed lockfile:

```sh
cargo audit --deny yanked
cargo test --locked --all-targets
```

CI checks every change and audits the default branch weekly. Vulnerability
advisories and yanked packages fail the audit. Do not add blanket advisory
ignores to make a release pass. Assess new findings, update affected dependencies,
and record any remaining platform/API exposure here before releasing.

## Assessment for 0.6.0

The refreshed lockfile has **no RustSec vulnerability advisories or yanked
packages** as of 2026-11-21. This is not a claim that dependencies are risk-free.

Remaining informational findings are kept visible in audit output:

- `atty` (RUSTSEC-2021-0145) can perform an unaligned read on Windows with a
  custom global allocator. This application uses Rust's default System allocator
  and does not install a custom allocator; the advisory states that Windows
  HeapAlloc provides sufficient alignment in this case. Reassess before adding
  an allocator. Remove atty by upgrading both the CLI parser and Unix picker
  dependency stacks in a separately tested change.
- Maintenance warnings remain for `ansi_term`, `atty`, `chrono`, `derivative`,
  `humantime`, `instant`, `pin-utils`, `proc-macro-error`, `structopt`,
  `zbus_names`, `zvariant`, and `zvariant_derive`. Plan migration away from the
  unmaintained parser/picker, date-time and keyring dependency stacks. The
  existing keyring backend is retained for credential compatibility; do not
  change it without upgrade tests on each supported OS.

Release builds pin the compiler and dependencies, use SHA-pinned GitHub Actions,
and smoke-test extracted archives on native runners. Checksums detect corruption;
they are not independent signatures or a reproducible-build guarantee.
