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
# The site's accent colour.
html_theme_options = {
    "light_css_variables": {"color-brand-primary": "#2070a8", "color-brand-content": "#2070a8"},
    "dark_css_variables": {"color-brand-primary": "#6fb3e6", "color-brand-content": "#6fb3e6"},
}
