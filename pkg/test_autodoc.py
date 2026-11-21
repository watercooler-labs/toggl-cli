"""Exercise failure rollback without invoking real Cargo or touching the checkout."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


@unittest.skipIf(os.name == "nt", "uses POSIX executable fixtures")
class AutodocTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.build = tempfile.TemporaryDirectory()
        cls.binary = Path(cls.build.name) / "autodoc"
        subprocess.run(["rustc", "--edition=2021", str(ROOT / "pkg/autodoc.rs"), "-o", str(cls.binary)], check=True, cwd=ROOT)

    @classmethod
    def tearDownClass(cls):
        cls.build.cleanup()

    def test_failed_build_or_help_rolls_back_all_files(self):
        for failing_phase in ("build", "help"):
            with self.subTest(phase=failing_phase), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                originals = {
                    "Cargo.toml": '[package]\nname = "fixture"\nversion = "0.5.1"\n',
                    "Cargo.lock": "original lock\n",
                    "README.md": "$ toggl help\nold\n```\n$ toggl help start\nold\n```\n",
                }
                for name, text in originals.items():
                    (root / name).write_text(text)
                scripts = {
                    "git": "#!/bin/sh\nexit 0\n",
                    "cargo": '#!/bin/sh\nprintf "changed lock\\n" > Cargo.lock\n'
                             + ('exit 37\n' if failing_phase == "build" else 'if [ "$1" = build ]; then exit 0; fi\nexit 37\n'),
                }
                for name, text in scripts.items():
                    (root / name).write_text(text)
                    (root / name).chmod(0o755)
                env = dict(os.environ, PATH=str(root) + os.pathsep + os.environ["PATH"])
                result = subprocess.run([str(self.binary), "minor"], cwd=root, env=env, capture_output=True)
                self.assertEqual(result.returncode, 1)
                self.assertIn(b"0.5.1 to 0.6.0", result.stdout)
                for name, text in originals.items():
                    self.assertEqual((root / name).read_text(), text)


if __name__ == "__main__":
    unittest.main()
