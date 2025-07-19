.PHONY: setup
setup:
	cp  .env-development .env
	cd dev-tools/pg && make setup
.PHONY: dev
dev:
	cargo watch --why --ignore '**/target/*' -s 'make dev-exec'

.PHONY: dev-exec
dev-exec:
	line-cli
	# clear
	cargo build
	# clear
	make test

.PHONY: test
test:
	cargo test -- --nocapture
