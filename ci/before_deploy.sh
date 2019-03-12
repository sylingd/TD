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

	names=',' read -r -a array <<< "$BIN_NAME"
	for THIS_BIN in "${names[@]}"
	do
		if [[ "$TARGET" =~ "windows" ]]; then
			cp target/$TARGET/release/$THIS_BIN.exe $stage1/
			cp target/$TARGET/debug/$THIS_BIN.exe $stage2/
		else
			cp target/$TARGET/release/$THIS_BIN $stage1/
			cp target/$TARGET/debug/$THIS_BIN $stage2/
		fi
	done

	cd $stage1
	zip $TRAVIS_BUILD_DIR/$PACKAGE_NAME-$TARGET-RELEASE.zip *
	cd $stage2
	zip $TRAVIS_BUILD_DIR/$PACKAGE_NAME-$TARGET-DEBUG.zip *
	cd $TRAVIS_BUILD_DIR

	git tag $RELEASE_NAME

	rm -rf $stage
}

main
