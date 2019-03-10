# This script takes care of building your crate and packaging it for release

main() {
	export RELEASE_NAME=`date +%m.%d`"."`echo $TRAVIS_COMMIT | cut -c 1-16`

	local stage=

	case $TRAVIS_OS_NAME in
		linux)
			stage=$(mktemp -d)
			;;
		osx)
			stage=$(mktemp -d -t tmp)
			;;
	esac

	if [[ "$TARGET" =~ "windows" ]]; then
		cp target/$TARGET/release/$CRATE_NAME.exe $stage/
	else
		cp target/$TARGET/release/$CRATE_NAME $stage/
	fi

	cd $stage
	zip $TRAVIS_BUILD_DIR/$CRATE_NAME-$TARGET.zip *
	cd $TRAVIS_BUILD_DIR

	git tag $RELEASE_NAME

	rm -rf $stage
}

main
