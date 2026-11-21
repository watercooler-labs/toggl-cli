#!/usr/bin/env python3
"""Package, verify and generate release metadata (Python 3.11+, stdlib only)."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parent.parent
TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
)
SHELLS = ("bash", "zsh", "fish", "powershell", "elvish")


def version():
    with (ROOT / "Cargo.toml").open("rb") as manifest:
        return tomllib.load(manifest)["package"]["version"]


def archive_name(target, release_version):
    return f"toggl-{target}-{release_version}.zip"


def checksum(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def smoke(binary, release_version):
    def run(*args):
        return subprocess.run([str(binary), *args], check=True, capture_output=True, timeout=30).stdout
    actual = run("--version").decode().strip()
    if actual != f"toggl {release_version}":
        raise ValueError(f"binary version mismatch: {actual!r}")
    run("--help")
    run("auth", "--profile", "release-smoke", "--help")
    run("list", "task", "--help")
    completions = {shell: run("completions", "--shell", shell) for shell in SHELLS}
    if not all(completions.values()):
        raise ValueError("empty completion script")
    return completions


def package(target, directory):
    release_version = version()
    binary_name = "toggl.exe" if "windows" in target else "toggl"
    target_root = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
    binary = target_root.resolve() / target / "release" / binary_name
    completions = smoke(binary, release_version)
    directory.mkdir(parents=True, exist_ok=True)
    archive = directory / archive_name(target, release_version)
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as output:
        output.write(binary, binary_name)
        output.write(ROOT / "LICENSE.md", "LICENSE.md")
        for shell, script in completions.items():
            output.writestr(f"completions/toggl.{shell}", script)
    # Execute the extracted artifact, not only the build-tree binary.
    with tempfile.TemporaryDirectory() as temp:
        with zipfile.ZipFile(archive) as source:
            source.extractall(temp)
        extracted = Path(temp) / binary_name
        if os.name != "nt":
            extracted.chmod(0o755)
        smoke(extracted, release_version)
    (directory / f"{target}.sha256").write_text(f"{checksum(archive)}  {archive.name}\n")
    (directory / "VERSION").write_text(f"{release_version}\n")
    return archive


def verify(directory, release_version):
    if (directory / "VERSION").read_text().strip() != release_version:
        raise ValueError("VERSION does not match the release tag/source")
    for target in TARGETS:
        name = archive_name(target, release_version)
        expected = f"{checksum(directory / name)}  {name}"
        if (directory / f"{target}.sha256").read_text().strip() != expected:
            raise ValueError(f"checksum mismatch for {name}")


def scoop_manifest(directory, release_version):
    verify(directory, release_version)
    archive = directory / archive_name("x86_64-pc-windows-msvc", release_version)
    template = (ROOT / "pkg/scoop/toggl-cli.json").read_text()
    manifest = json.loads(template.replace("$APP_VERSION", release_version).replace("$SHA256_HASH", checksum(archive)))
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("version", "package", "verify", "scoop"))
    parser.add_argument("--target", choices=TARGETS)
    parser.add_argument("--directory", type=Path, default=Path("dist"))
    parser.add_argument("--tag")
    args = parser.parse_args()
    release_version = version()
    if args.tag is not None and args.tag != f"v{release_version}":
        parser.error(f"tag must match Cargo.toml: v{release_version}")
    if args.command == "version":
        print(release_version)
    elif args.command == "package":
        if args.target is None:
            parser.error("package requires --target")
        print(package(args.target, args.directory))
    elif args.command == "verify":
        verify(args.directory, release_version)
    else:
        destination = args.directory / "toggl-cli.json"
        destination.write_text(json.dumps(scoop_manifest(args.directory, release_version), indent=2) + "\n")
        print(destination)


if __name__ == "__main__":
    main()
