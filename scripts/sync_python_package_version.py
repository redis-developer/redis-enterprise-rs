#!/usr/bin/env python3
"""Keep the Python package version aligned with the root crate version."""

from __future__ import annotations

from pathlib import Path
import re
import tomllib


ROOT = Path(__file__).resolve().parent.parent
ROOT_CARGO = ROOT / "Cargo.toml"
PYTHON_CARGO = ROOT / "python" / "Cargo.toml"
VERSION_RE = re.compile(r'^(version = )"[^"]+"$', re.MULTILINE)


def read_root_version() -> str:
    with ROOT_CARGO.open("rb") as fh:
        data = tomllib.load(fh)
    return data["package"]["version"]


def sync_python_version(version: str) -> None:
    original = PYTHON_CARGO.read_text()
    updated, replacements = VERSION_RE.subn(rf'\1"{version}"', original, count=1)
    if replacements != 1:
        raise RuntimeError("failed to update python/Cargo.toml version")
    PYTHON_CARGO.write_text(updated)


def main() -> None:
    sync_python_version(read_root_version())


if __name__ == "__main__":
    main()
