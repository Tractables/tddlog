"""The pages served at tractables.github.io/tddlog/c/: the library's
README, its example, and the header it documents."""

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
manifest = (ROOT / "bindings/c/Cargo.toml").read_text(encoding="utf-8")
project = "tddlog for C"
release = re.search(r'^version = "([^"]+)"', manifest, re.MULTILINE).group(1)
extensions = ["myst_parser"]
html_theme = "furo"
html_title = "tddlog for C"
html_logo = str(ROOT / "docs/logo.svg")
html_favicon = "https://tractables.github.io/tddlog/favicon.svg"
# The theme the reader chose on the site's other pages, and the bar of
# links below (bindings/sphinx).
html_static_path = [str(ROOT / "bindings/sphinx")]
html_js_files = ["site-theme.js"]
html_css_files = ["site.css"]
# The site's links to its pages, as the other pages list them, in a bar
# above the page (this page's name not a link; on a phone's width the bar
# wraps between links), and its colours.
SITE = [("Playground", "../"), ("Guide", "../guide/"), ("Results", "../results/"), ("Rust API", "../api/"),
        ("Python", "../python/"), ("C", None), ("GitHub", "https://github.com/Tractables/tddlog")]
LINKS = [f'<a href="{href}">{name}</a>' if href else f"<b>{name}</b>" for name, href in SITE]
html_theme_options = {
    "announcement": '<span style="font-size: 0.875em">'
    + " ".join(f'<span style="white-space: nowrap">{link} ·</span>' for link in LINKS[:-1])
    + f" {LINKS[-1]}</span>",
    "light_css_variables": {"color-brand-primary": "#2070a8", "color-brand-content": "#2070a8",
                            "color-announcement-background": "#f5f5f2", "color-announcement-text": "#2070a8"},
    "dark_css_variables": {"color-brand-primary": "#6fb3e6", "color-brand-content": "#6fb3e6",
                           "color-announcement-background": "#16181b", "color-announcement-text": "#6fb3e6"},
}
