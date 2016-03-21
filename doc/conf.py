# -*- coding: utf-8 -*-

extensions = []
templates_path = ['_templates']
source_suffix = '.rst'
master_doc = 'index'

project = u'Rotor'
copyright = u'2015, Paul Colomiets'

version = '0.6'
release = '0.6.3'
exclude_patterns = ['_build']
pygments_style = 'sphinx'
html_theme = 'default'
html_static_path = ['_static']
htmlhelp_basename = 'Rotordoc'

latex_elements = { }

latex_documents = [
  ('index', 'Rotor.tex', u'Rotor Documentation',
   u'Paul Colomiets', 'manual'),
]

man_pages = [
    ('index', 'rotor', u'Rotor Documentation',
     [u'Paul Colomiets'], 1)
]

texinfo_documents = [
  ('index', 'Rotor', u'Rotor Documentation',
   u'Paul Colomiets', 'Rotor', 'Asynchronous I/O for rust.',
   'Miscellaneous'),
]

# on_rtd is whether we are on readthedocs.org
import os
on_rtd = os.environ.get('READTHEDOCS', None) == 'True'

if not on_rtd:  # only import and set the theme if we're building docs locally
    import sphinx_rtd_theme
    html_theme = 'sphinx_rtd_theme'
    html_theme_path = [sphinx_rtd_theme.get_html_theme_path()]

# otherwise, readthedocs.org uses their theme by default, so no need to specify it
