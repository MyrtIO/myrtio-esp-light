#!/bin/bash

html-minifier-terser assets/factory_page.html \
--collapse-whitespace \
--remove-comments \
--remove-optional-tags \
--remove-redundant-attributes \
--remove-script-type-attributes \
--remove-style-link-type-attributes \
--minify-css true \
--minify-js true \
| gzip -9 > assets/factory_page.html.min.gz