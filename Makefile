
build:
	./build.sh alpha_backend ./src/alpha_backend/alpha_backend.did


release:
	docker build -t alpha_backend .
	mkdir -p $(shell pwd)/release-artifacts
	docker run --rm -v $(shell pwd)/release-artifacts:/target/wasm32-unknown-unknown/release alpha_backend
	shasum -a 256 $(shell pwd)/release-artifacts/alpha_backend.wasm  | cut -d ' ' -f 1
