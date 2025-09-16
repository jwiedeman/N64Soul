#!/usr/bin/env python3
"""Validate that the Python environment has the modules required for exports."""

from __future__ import annotations

import importlib
import sys
from typing import Iterable, Optional, Tuple

REQUIRED: Tuple[Tuple[str, Optional[str]], ...] = (
    ("torch", "pip install torch --extra-index-url https://download.pytorch.org/whl/cpu"),
    ("transformers", "pip install transformers"),
)

OPTIONAL: Tuple[Tuple[str, Optional[str]], ...] = (
    ("numpy", None),
    ("sentencepiece", "pip install sentencepiece"),
)


def _probe_module(name: str) -> Tuple[Optional[str], Optional[str]]:
    """Return (status, detail).

    status is one of:
        "ok"      -> detail is the version string
        "missing" -> module not found; detail is None
        "error"   -> import raised an unexpected exception; detail is the message
    """

    try:
        module = importlib.import_module(name)
    except ModuleNotFoundError:
        return ("missing", None)
    except Exception as exc:  # pragma: no cover - defensive logging
        return ("error", str(exc))

    version = getattr(module, "__version__", "unknown")
    return ("ok", str(version))


def _report_category(items: Iterable[Tuple[str, Optional[str]]], *, optional: bool = False) -> Tuple[list, list]:
    missing = []
    errors = []
    for name, hint in items:
        status, detail = _probe_module(name)
        if status == "ok":
            print(f"[ok] {name} {detail}")
        elif status == "missing":
            prefix = "[optional]" if optional else "[missing]"
            print(f"{prefix} {name} not found")
            if not optional:
                missing.append((name, hint))
        else:
            prefix = "[optional-error]" if optional else "[error]"
            print(f"{prefix} {name}: {detail}")
            if not optional:
                errors.append((name, detail))
    return missing, errors


def main() -> int:
    missing, errors = _report_category(REQUIRED)
    _report_category(OPTIONAL, optional=True)

    if missing or errors:
        print("\nPython dependencies missing:")
        for name, hint in missing:
            if hint:
                print(f" - {name}: {hint}")
            else:
                print(f" - {name}: pip install {name}")
        for name, detail in errors:
            print(f" - {name}: import error: {detail}")
        print("\nInstall the packages above before running the export scripts.")
        return 1

    print("All required Python modules are available.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
