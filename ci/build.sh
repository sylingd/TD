main() {
	cd $TRAVIS_BUILD_DIR

	cross build --target $TARGET
	cross build --target $TARGET --release

	# cross test --target $TARGET
	# cross test --target $TARGET --release

	# cross run --target $TARGET
	# cross run --target $TARGET --release
}

main
