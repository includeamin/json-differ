
# format code
fmt:
	@echo "format code"
	@cargo fmt --all

# bump dependencies
bump:
	@echo "bump dependencies"
	@cargo update

# ignore .idea
lint:
	@echo "lint code"
	@cargo clippy --all-targets --all-features -- -D warnings

lint-fix:
	@echo "lint fix code"
	@cargo clippy --all-targets --allow-dirty --allow-staged --all-features --fix -- -D warnings

test:
	@echo "test code"
	@cargo test --all-targets --all-features
