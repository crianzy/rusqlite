ARCHS = x86_64-apple-ios armv7s-apple-ios aarch64-apple-ios armv7-apple-ios

$(ARCHS): %:
	cargo build --features "sqlcipher bundled optimize" --release --target $@

clean:
	rm target/*-ios/release/build/*/out/libsqlite3.a

fat: clean $(ARCHS)
	lipo -create -output libsqlite3.a target/*-ios/release/build/*/out/libsqlite3.a
