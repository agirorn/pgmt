.PHONY: dev
dev:
	cargo watch -s 'make dev-exec'

.PHONY: dev-exec
dev-exec:
	clear
	cargo build
	make test

.PHONY: test
test:
	cargo test -- --nocapture
