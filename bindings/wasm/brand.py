#!/usr/bin/env python3
"""Brand the assembled site's pages: the project logo above each page's
heading, and in each page's head its icon, the theme's colour for the
browser's frame, and for a page of its own the address, title, description
and picture that a link to it shows where it is shared."""

import html
import os
from pathlib import Path
import re
import sys

# Where GitHub Pages serves the site (.github/workflows/pages.yml).
URL = "https://tractables.github.io/tddlog/"
# The pages that are addresses of their own; 404.html, which GitHub Pages
# serves at any address the site does not have, has the logo and the icon.
PAGES = ["index.html", "guide/index.html", "results/index.html"]
# The page's background in either theme (styles.css's --bg).
THEME_COLOURS = {"light": "#f5f5f2", "dark": "#16181b"}


def attribute(text):
    """`text`, as written in a page, as an attribute's value."""
    return html.escape(html.unescape(text.strip()), quote=True)


def head(page, content, root, version):
    tags = [f'<link rel="icon" type="image/svg+xml" href="{root}/favicon.svg{version}">']
    tags += [f'<meta name="theme-color" content="{colour}" media="(prefers-color-scheme: {scheme})">'
             for scheme, colour in THEME_COLOURS.items()]
    if page in PAGES:
        title = re.search(r"<title>(.*?)</title>", content, re.S)
        description = re.search(r'<meta name="description" content="([^"]*)">', content)
        if title is None or description is None:
            raise ValueError(f"{page}: expected a title and a description")
        address = URL + page.removesuffix("index.html")
        tags += [
            f'<link rel="canonical" href="{address}">',
            '<meta property="og:type" content="website">',
            '<meta property="og:site_name" content="tddlog">',
            f'<meta property="og:title" content="{attribute(title.group(1))}">',
            f'<meta property="og:description" content="{attribute(description.group(1))}">',
            f'<meta property="og:url" content="{address}">',
            f'<meta property="og:image" content="{URL}social.png">',
            '<meta property="og:image:width" content="1200">',
            '<meta property="og:image:height" content="630">',
            '<meta property="og:image:alt" content="The tddlog logo and its tagline">',
            '<meta name="twitter:card" content="summary_large_image">',
        ]
    return "\n".join(tags) + "\n"


def brand(site, stamp=""):
    site = Path(site).resolve()
    version = f"?v={stamp}" if stamp else ""
    for page in PAGES + ["404.html"]:
        path = site / page
        root = Path(os.path.relpath(site, path.parent)).as_posix()
        mark = (f'<a class="site-brand" href="{root}/" aria-label="tddlog playground">'
                f'<img class="site-logo" src="{html.escape(f"{root}/logo.svg{version}", quote=True)}" '
                'alt="tddlog" width="720" height="365"></a>\n    ')
        content = path.read_text(encoding="utf-8")
        heading = re.compile(r"<h1(?=[\s>])")
        if not heading.search(content) or "</head>" not in content or 'class="site-brand"' in content:
            raise ValueError(f"{path}: expected an unbranded page with a head and a heading")
        content = content.replace("</head>", head(page, content, root, version) + "</head>", 1)
        path.write_text(heading.sub(lambda m: mark + m.group(0), content, count=1), encoding="utf-8")


if __name__ == "__main__":
    brand(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else "")
