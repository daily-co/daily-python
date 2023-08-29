# Configuration for the Sphinx documentation builder.
# https://www.sphinx-doc.org/en/master/usage/configuration.html

import os
import sys

import toml

def get_release_version() -> str:
    """Get the release version from the Cargo.toml file.

    :return:
    """
    cargo_content = toml.load("../../Cargo.toml")
    return cargo_content["package"]["version"]


# Project
project = "daily-python"
copyright = "2023 Daily"
version = get_release_version()


# General

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    "sphinx.ext.autodoc",
]

autodoc_typehints = "description"
# nitpicky = True
# nitpick_ignore = [
#     ("py:class", "pyarrow.lib.Schema"),
#     ("py:class", "pyarrow._dataset.Dataset"),
#     ("py:class", "pyarrow.lib.Table"),
#     ("py:class", "pyarrow.lib.DataType"),
#     ("py:class", "pyarrow.lib.Field"),
#     ("py:class", "pyarrow.lib.NativeFile"),
#     ("py:class", "pyarrow.lib.RecordBatchReader"),
#     ("py:class", "pyarrow._fs.FileSystem"),
#     ("py:class", "pyarrow._fs.FileInfo"),
#     ("py:class", "pyarrow._fs.FileSelector"),
#     ("py:class", "pyarrow._fs.FileSystemHandler"),
#     ("py:class", "RawDeltaTable"),
#     ("py:class", "pandas.DataFrame"),
#     ("py:class", "pyarrow._dataset_parquet.ParquetFileWriteOptions"),
#     ("py:class", "pathlib.Path"),
#     ("py:class", "datetime.datetime"),
#]

# Add any paths that contain templates here, relative to this directory.
templates_path = ["_templates"]

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = []

# HTML output

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
html_theme = "sphinx_rtd_theme"

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = []

html_favicon = "favicon.ico"

# Don't show "Video page source" link
html_show_sourcelink = False
