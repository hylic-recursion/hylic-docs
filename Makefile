.PHONY: test test-ci hylic-docs-build hylic-docs-serve hylic-docs-open

test:
	@bash Makefile-scripting/test.sh

test-ci:
	@bash Makefile-scripting/test-ci.sh

hylic-docs-build:
	@cd book && mdbook build

hylic-docs-serve: hylic-docs-build
	@echo "Serving at http://localhost:8321/"
	@cd target/book && python3 -m http.server 8321

hylic-docs-open: hylic-docs-build
	@xdg-open target/book/index.html

# backwards compat
book: hylic-docs-build
open: hylic-docs-open
