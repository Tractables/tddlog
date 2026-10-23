#!/usr/bin/env python3
"""Add the project logo to the assembled playground, guide and results pages."""

import html
import os
from pathlib import Path
import shutil
import sys


def brand(site, stamp=""):
    site = Path(site).resolve()
    source = Path(__file__).resolve().parents[2] / "docs/logo.svg"
    shutil.copyfile(source, site / "logo.svg")
    pages = [site / "index.html", site / "guide/index.html", site / "results/index.html"]
    pages.extend(sorted((site / "results/benchmarks").rglob("*.html")))
    for page in pages:
        root = Path(os.path.relpath(site, page.parent)).as_posix()
        logo = f"{root}/logo.svg" + (f"?v={stamp}" if stamp else "")
        mark = (f'<a class="site-brand" href="{root}/" aria-label="tddlog playground">'
                f'<img class="site-logo" src="{html.escape(logo, quote=True)}" '
                'alt="tddlog" width="720" height="365"></a>\n    ')
        content = page.read_text(encoding="utf-8")
        if '<h1>' not in content or 'class="site-brand"' in content:
            raise ValueError(f"{page}: expected an unbranded page with a heading")
        page.write_text(content.replace('<h1>', mark + '<h1>', 1), encoding="utf-8")


if __name__ == "__main__":
    brand(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else "")
