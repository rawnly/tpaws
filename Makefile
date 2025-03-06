release:
	. .env
	@cargo build --release
	@tar -czf tpaws.tar.gz --directory=./target/release tpaws
	@shasum -a 256 tpaws.tar.gz | cut -d ' ' -f1 | xargs -I{} sd '\{\{shasum\}\}' '{}' tpaws.rb

install:
	. .env && TPAWS_COMMIT_ID=$$(git rev-parse HEAD); \
	cargo install --path .

