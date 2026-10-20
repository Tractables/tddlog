"""The page served at tractables.github.io/tddlog/python/: the package's
README, then the reference read from the docstrings of the installed
package."""

import tddlog

project = "tddlog for Python"
release = tddlog.__version__
extensions = ["myst_parser", "sphinx.ext.autodoc"]
html_theme = "furo"
html_title = "tddlog for Python"
# The site's accent colour.
html_theme_options = {
    "light_css_variables": {"color-brand-primary": "#2070a8", "color-brand-content": "#2070a8"},
    "dark_css_variables": {"color-brand-primary": "#6fb3e6", "color-brand-content": "#6fb3e6"},
}
autodoc_member_order = "bysource"
