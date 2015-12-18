all:
	cargo build


notes:
	@clang notes.c -o notes
	@./notes
	@rm notes
