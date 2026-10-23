"""The release metadata check (.github/scripts/check_release.py): it accepts
this checkout and its tag, and refuses a stale version in any one file."""

from pathlib import Path
import shutil
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / ".github" / "scripts"))
from check_release import check  # noqa: E402

FILES = ("Cargo.toml", "Cargo.lock", "CITATION.cff", "README.md",
         "bindings/python/Cargo.toml", "bindings/python/pyproject.toml",
         "bindings/c/Cargo.toml", "bindings/c/CMakeLists.txt", "bindings/wasm/Cargo.toml")


class ReleaseTests(unittest.TestCase):
    def test_checkout_and_tag(self):
        version = check(ROOT)
        self.assertEqual(check(ROOT, "v" + version), version)
        with self.assertRaisesRegex(ValueError, "release tag"):
            check(ROOT, "v999.0.0")

    def test_each_file_detects_a_stale_version(self):
        version = check(ROOT)
        # How each file goes stale: its version replaced by another; for the
        # files that take the version from a Cargo manifest, a version of
        # their own; for the README, a citation of another version.
        def replace(old, new):
            def stale(text):
                self.assertIn(old, text)
                return text.replace(old, new)
            return stale
        stale = {
            "bindings/python/pyproject.toml": replace('dynamic = ["version"]', f'version = "{version}"'),
            "bindings/c/CMakeLists.txt": replace("VERSION ${TDDLOG_VERSION}", f"VERSION {version}"),
            "README.md": lambda text: text + "\n  note = {version 999.0.0}\n",
        }
        for relative in FILES:
            with self.subTest(file=relative), tempfile.TemporaryDirectory() as directory:
                source = Path(directory)
                for name in FILES:
                    (source / name).parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(ROOT / name, source / name)
                changed = source / relative
                edit = stale.get(relative, replace(version, "999.0.0"))
                changed.write_text(edit(changed.read_text(encoding="utf-8")), encoding="utf-8")
                with self.assertRaises(ValueError):
                    check(source)


if __name__ == "__main__":
    unittest.main()
