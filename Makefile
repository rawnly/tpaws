release:
	. .env
	cargo build --release
	tar -czf tpaws.tar.gz --directory=./target/release tpaws
	SHASUM=$$(shasum -a 256 tpaws.tar.gz | cut -d ' ' -f 1) ; \
	sd '\{\{shasum\}\}' "$$SHASUM" tpaws.rb

install:
	. .env && TPAWS_COMMIT_ID=$$(git rev-parse HEAD); \
	cargo install --path .

