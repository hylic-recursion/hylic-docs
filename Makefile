.PHONY: test test-ci hylic-docs-check-anchors hylic-docs-build hylic-docs-serve hylic-docs-open

test:
	@bash Makefile-scripting/test.sh

test-ci:
	@bash Makefile-scripting/test-ci.sh

hylic-docs-check-anchors:
	@echo "Checking {{#include}} anchors..."
	@bash -c '\
	errfile=$$(mktemp); \
	grep -rn "{{#include" book/src/ --include="*.md" | while IFS= read -r line; do \
		mdfile=$$(echo "$$line" | cut -d: -f1); \
		mddir=$$(dirname "$$mdfile"); \
		include=$$(echo "$$line" | sed "s/.*{{#include \(.*\)}}.*/\1/"); \
		ref=$$(echo "$$include" | cut -d: -f1); \
		anchor=$$(echo "$$include" | grep ":" | sed "s/[^:]*://" | grep -v "^[0-9]"); \
		resolved=$$(cd "$$mddir" && realpath -m "$$ref" 2>/dev/null); \
		if [ ! -f "$$resolved" ]; then \
			echo "ERROR [$$mdfile]: file not found: $$ref" | tee -a "$$errfile"; \
		elif [ -n "$$anchor" ] && ! grep -q "ANCHOR: $$anchor" "$$resolved" 2>/dev/null; then \
			echo "ERROR [$$mdfile]: anchor \"$$anchor\" not found in $$resolved" | tee -a "$$errfile"; \
		fi; \
	done; \
	if [ -s "$$errfile" ]; then rm "$$errfile"; exit 1; fi; \
	rm "$$errfile"; \
	echo "All anchors OK."'

hylic-docs-build: hylic-docs-check-anchors
	@cd book && mdbook build

hylic-docs-serve: hylic-docs-build
	@fuser -k 8321/tcp 2>/dev/null || true
	@cd target/book && nohup python3 -m http.server 8321 > /dev/null 2>&1 &
	@echo "Serving at http://localhost:8321/"

hylic-docs-open: hylic-docs-build
	@xdg-open target/book/index.html

# backwards compat
book: hylic-docs-build
open: hylic-docs-open
