"""The page served at tractables.github.io/tddlog/c/: the library's README,
then the header it documents."""

import re
from pathlib import Path

manifest = (Path(__file__).resolve().parents[1] / "Cargo.toml").read_text(encoding="utf-8")
project = "tddlog for C"
release = re.search(r'^version = "([^"]+)"', manifest, re.MULTILINE).group(1)
extensions = ["myst_parser"]
html_theme = "furo"
html_title = "tddlog for C"
html_logo = str(Path(__file__).resolve().parents[3] / "docs/logo.svg")
# The site's links to its pages, as the other pages list them, in a bar
# above the page (this page's name not a link, the text small enough for a
# phone's width), and its colours.
SITE = [("Playground", "../"), ("Guide", "../guide/"), ("Results", "../results/"), ("API", "../api/"),
        ("Python", "../python/"), ("C", None), ("GitHub", "https://github.com/Tractables/tddlog")]
html_theme_options = {
    "announcement": '<span style="font-size: 0.875em">'
    + " · ".join(f'<a href="{href}">{name}</a>' if href else f"<b>{name}</b>" for name, href in SITE) + "</span>",
    "light_css_variables": {"color-brand-primary": "#2070a8", "color-brand-content": "#2070a8",
                            "color-announcement-background": "#f5f5f2", "color-announcement-text": "#2070a8"},
    "dark_css_variables": {"color-brand-primary": "#6fb3e6", "color-brand-content": "#6fb3e6",
                           "color-announcement-background": "#16181b", "color-announcement-text": "#6fb3e6"},
}
