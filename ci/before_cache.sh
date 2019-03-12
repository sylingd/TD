main() {
	names=',' read -r -a array <<< "$BIN_NAME"
	cd $TRAVIS_BUILD_DIR
	for THIS_BIN in "${names[@]}"
	do
		rm -rfv target/$TARGET/debug/incremental/$THIS_BIN-*
		rm -rfv target/$TARGET/debug/.fingerprint/$THIS_BIN-*
		rm -rfv target/$TARGET/debug/build/$THIS_BIN-*
		rm -rfv target/$TARGET/debug/deps/$THIS_BIN-*
		rm -rfv target/$TARGET/debug/$THIS_BIN.d
		rm -rfv target/$TARGET/release/incremental/$THIS_BIN-*
		rm -rfv target/$TARGET/release/.fingerprint/$THIS_BIN-*
		rm -rfv target/$TARGET/release/build/$THIS_BIN-*
		rm -rfv target/$TARGET/release/deps/$THIS_BIN-*
		rm -rfv target/$TARGET/release/$THIS_BIN.d
		cargo clean -p $THIS_BIN
	done
}