# This script takes care of building your crate and packaging it for release

main() {
	local stage1=
	local stage2=

	case $TRAVIS_OS_NAME in
		linux)
			stage1=$(mktemp -d)
			stage2=$(mktemp -d)
			;;
		osx)
			stage1=$(mktemp -d -t tmp)
			stage2=$(mktemp -d -t tmp)
			;;
	esac

	if [[ "$TARGET" =~ "windows" ]]; then
		cp target/$TARGET/release/$CRATE_NAME.exe $stage1/
		cp target/$TARGET/debug/$CRATE_NAME.exe $stage2/
	else
		cp target/$TARGET/release/$CRATE_NAME $stage1/
		cp target/$TARGET/debug/$CRATE_NAME $stage2/
	fi

	cd $stage1
	zip $TRAVIS_BUILD_DIR/$CRATE_NAME-$TARGET-RELEASE.zip *
	cd $stage2
	zip $TRAVIS_BUILD_DIR/$CRATE_NAME-$TARGET-DEBUG.zip *
	cd $TRAVIS_BUILD_DIR

	git tag $RELEASE_NAME

	rm -rf $stage
}

main
