main() {
	cd $TRAVIS_BUILD_DIR
	bins=(${BIN_NAME//,/ })
	for bin in "${bins[@]}"
	do
		rm -rfv target/$TARGET/debug/incremental/$bin-*
		rm -rfv target/$TARGET/debug/.fingerprint/$bin-*
		rm -rfv target/$TARGET/debug/build/$bin-*
		rm -rfv target/$TARGET/debug/deps/$bin-*
		rm -rfv target/$TARGET/debug/$bin.d
		rm -rfv target/$TARGET/release/incremental/$bin-*
		rm -rfv target/$TARGET/release/.fingerprint/$bin-*
		rm -rfv target/$TARGET/release/build/$bin-*
		rm -rfv target/$TARGET/release/deps/$bin-*
		rm -rfv target/$TARGET/release/$bin.d
	done
	cargo clean -p $PACKAGE_NAME
}

main