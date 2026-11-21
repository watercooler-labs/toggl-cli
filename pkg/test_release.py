import json
from pathlib import Path
import tempfile
import unittest

import release


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.version = "0.10.12"
        (self.directory / "VERSION").write_text(self.version + "\n")
        for target in release.TARGETS:
            archive = self.directory / release.archive_name(target, self.version)
            archive.write_bytes(b"fixture binary archive")
            (self.directory / f"{target}.sha256").write_text(f"{release.checksum(archive)}  {archive.name}\n")

    def test_verifies_complete_artifact_set(self):
        release.verify(self.directory, self.version)

    def test_rejects_corruption_missing_targets_and_version_mismatch(self):
        with self.assertRaises(ValueError):
            release.verify(self.directory, "0.10.13")
        target = release.TARGETS[0]
        archive = self.directory / release.archive_name(target, self.version)
        archive.write_bytes(b"corrupted")
        with self.assertRaises(ValueError):
            release.verify(self.directory, self.version)
        archive.unlink()
        with self.assertRaises(FileNotFoundError):
            release.verify(self.directory, self.version)

    def test_scoop_uses_current_msvc_asset_version_and_hash(self):
        manifest = release.scoop_manifest(self.directory, self.version)
        package = manifest["architecture"]["64bit"]
        self.assertEqual(manifest["version"], self.version)
        archive = self.directory / release.archive_name("x86_64-pc-windows-msvc", self.version)
        self.assertTrue(package["url"].endswith(f"/v{self.version}/{archive.name}"))
        self.assertEqual(package["hash"], "sha256:" + release.checksum(archive))
        self.assertNotIn("$", json.dumps(manifest))


if __name__ == "__main__":
    unittest.main()
