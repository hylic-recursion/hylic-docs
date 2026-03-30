.PHONY: test book open

test:
	@bash Makefile-scripting/test.sh

book:
	@bash Makefile-scripting/build-book.sh

open: book
	@xdg-open target/book/index.html
