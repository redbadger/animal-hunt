
build-image:
	cd .. && tar -cf context.tar shared server Cargo.toml Cargo.lock rust-toolchain.toml
	cd .. && docker build --file server/Dockerfile - < context.tar

deploy:
	flyctl deploy .. --config server/fly.toml
