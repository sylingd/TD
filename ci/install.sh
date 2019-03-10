set -ex

say() {
	echo "install.sh: $1"
}

main() {
    local target=
    if [ $TRAVIS_OS_NAME = linux ]; then
        target=x86_64-unknown-linux-musl
        sort=sort
    else
        target=x86_64-apple-darwin
        sort=gsort  # for `sort --sort-version`, from brew's coreutils.
    fi

	git="rust-embedded/cross"
	url="https://github.com/$git"
	say "GitHub repository: $url"

	if [ -z $crate ]; then
		crate=$(echo $git | cut -d'/' -f2)
	fi

	say "Crate: $crate"

	url="$url/releases"

	if [ -z $tag ]; then
		tag=$(curl -s "$url/latest" | cut -d'"' -f2 | rev | cut -d'/' -f1 | rev)
		say "Tag: latest ($tag)"
	else
		say "Tag: $tag"
	fi

	if [ -z $target ]; then
		target=$(rustc -Vv | grep host | cut -d' ' -f2)
	fi

	say "Target: $target"

	if [ -z $dest ]; then
		dest="$HOME/.cargo/bin"
	fi

	say "Installing to: $dest"

	url="$url/download/$tag/$crate-$tag-$target.tar.gz"

	td=$(mktemp -d || mktemp -d -t tmp)
	curl -sL $url | tar -C $td -xz

	for f in $(ls $td); do
		test -x $td/$f || continue

		if [ -e "$dest/$f" ] && [ $force = false ]; then
			say "$f already exists in $dest"
		else
			mkdir -p $dest
			install -m 755 $td/$f $dest
		fi
	done

	rm -rf $td
}

main
