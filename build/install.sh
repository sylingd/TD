set -ex

install_c_toolchain() {
    case $TARGET in
        aarch64-unknown-linux-gnu)
            sudo apt-get install -y --force-yes --no-install-recommends \
                 gcc-aarch64-linux-gnu libc6-arm64-cross libc6-dev-arm64-cross
            ;;
        *)
            # For other targets, this is handled by addons.apt.packages in .travis.yml
            ;;
    esac
}

install_rustup() {
  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=beta

  rustc -V
  cargo -V
}

install_standard_crates() {
  rustup target add $TARGET
}

configure_cargo() {
  local prefix

  case "$TARGET" in
    aarch64-unknown-linux-gnu)
      prefix=aarch64-linux-gnu
      ;;
    x86_64-pc-windows-gnu)
      prefix=x86_64-w64-mingw32
      ;;
    *)
      return
      ;;
  esac

  mkdir -p ~/.cargo

  cat >>~/.cargo/config <<EOF
[target.$TARGET]
linker = "$prefix-gcc"
EOF
}

main() {
  install_c_toolchain
  install_rustup
  install_standard_crates
  configure_cargo
}

main
