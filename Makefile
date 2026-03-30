.PHONY: test test-ci book open

test:
	@bash Makefile-scripting/test.sh

test-ci:
	@bash Makefile-scripting/test-ci.sh

book:
	@bash Makefile-scripting/build-book.sh

open: book
	@xdg-open target/book/index.html
