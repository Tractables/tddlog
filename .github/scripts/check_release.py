#!/usr/bin/env python3
"""Check that the crate, its bindings and the release metadata declare one
version, and that a release tag, where given, names it.

    python3 .github/scripts/check_release.py [--release-tag vX.Y.Z]

A tag whose version had not been bumped would build wheels labelled with
the previous release and publish them to PyPI, where a version can be
yanked but never replaced.
"""

import argparse
from pathlib import Path
import re

# The crate and its three bindings, each a package of its own.
MANIFESTS = ("Cargo.toml", "bindings/python/Cargo.toml", "bindings/c/Cargo.toml",
             "bindings/wasm/Cargo.toml")


def one(text, pattern, source):
    matches = re.findall(pattern, text, re.M)
    if len(matches) != 1:
        raise ValueError(f"{source}: expected one version declaration, found {len(matches)}")
    return matches[0]


def check(root, tag=""):
    """The version every file declares; ValueError where one differs."""
    read = lambda relative: (root / relative).read_text(encoding="utf-8")
    version = one(read("Cargo.toml"), r'^version = "([^"]+)"$', "Cargo.toml")
    if tag and tag != f"v{version}":
        raise ValueError(f"release tag {tag} differs from v{version}")
    for relative in MANIFESTS[1:]:
        found = one(read(relative), r'^version = "([^"]+)"$', relative)
        if found != version:
            raise ValueError(f"{relative}: version {found}, expected {version}")
    locked = [one(package, r'^version = "([^"]+)"$', "Cargo.lock")
              for package in read("Cargo.lock").split("[[package]]")[1:]
              if re.search(r'^name = "tddlog"$', package, re.M)]
    if locked != [version]:
        raise ValueError(f"Cargo.lock: tddlog is {', '.join(locked) or 'missing'}, expected {version}")
    citation = one(read("CITATION.cff"), r'^version:\s*["\']?([^"\'\s]+)["\']?\s*$', "CITATION.cff")
    if citation != version:
        raise ValueError(f"CITATION.cff: version {citation}, expected {version}")
    # The README's BibTeX entry, where it names a version.
    for found in re.findall(r'\bversion (\d+\.\d+\.\d+[^\s}]*)\}', read("README.md")):
        if found != version:
            raise ValueError(f"README.md: citation names version {found}, expected {version}")
    # maturin and CMake take the version from the bindings' Cargo manifests.
    pyproject = read("bindings/python/pyproject.toml")
    if not re.search(r'^dynamic = \["version"\]$', pyproject, re.M) or re.search(r'^version\s*=', pyproject, re.M):
        raise ValueError("bindings/python/pyproject.toml: the version must come from bindings/python/Cargo.toml")
    cmake = one(read("bindings/c/CMakeLists.txt"), r'^project\(tddlog_c VERSION (\S+)', "bindings/c/CMakeLists.txt")
    if cmake != "${TDDLOG_VERSION}":
        raise ValueError("bindings/c/CMakeLists.txt: the version must come from bindings/c/Cargo.toml")
    return version


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--release-tag", default="", help="the tag about to be released, such as v0.1.0")
    args = parser.parse_args()
    try:
        print(f"release metadata agrees: {check(Path(__file__).resolve().parents[2], args.release_tag)}")
    except ValueError as error:
        parser.exit(1, f"{error}\n")
