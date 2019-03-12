# This script takes care of building your crate and packaging it for release

main() {
	local stage=

	case $TRAVIS_OS_NAME in
		linux)
			stage=$(mktemp -d)
			;;
		osx)
			stage=$(mktemp -d -t tmp)
			;;
	esac

	mkdir $stage/$PACKAGE_NAME-release
	mkdir $stage/$PACKAGE_NAME-debug

	bins=(${BIN_NAME//,/ })
	for bin in "${bins[@]}"
	do
		if [[ "$TARGET" =~ "windows" ]]; then
			cp target/$TARGET/release/$bin.exe $stage/$PACKAGE_NAME-release/
			cp target/$TARGET/debug/$bin.exe $stage/$PACKAGE_NAME-debug/
		else
			cp target/$TARGET/release/$bin $stage/$PACKAGE_NAME-release/
			cp target/$TARGET/debug/$bin $stage/$PACKAGE_NAME-debug/
		fi
	done

	cd $stage
	zip $TRAVIS_BUILD_DIR/$PACKAGE_NAME-$TARGET-RELEASE.zip $PACKAGE_NAME-release/*
	zip $TRAVIS_BUILD_DIR/$PACKAGE_NAME-$TARGET-DEBUG.zip $PACKAGE_NAME-debug/*

	cd $TRAVIS_BUILD_DIR
	git tag $RELEASE_NAME

	rm -rf $stage
}

main
