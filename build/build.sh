set -ex

main() {
  cd $TRAVIS_BUILD_DIR

  cargo build --release --target $TARGET

  if [[ "$TARGET" =~ "windows" ]]; then
    file target/$TARGET/release/$CRATE_NAME.exe
  else
    file target/$TARGET/release/$CRATE_NAME
  fi
}

main
