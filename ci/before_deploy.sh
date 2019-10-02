# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cross rustc --bin csrf-cookie-token --target $TARGET --release -- -C lto

    TARGET_NAME=$(echo $TARGET | sed 's/-unknown//g')
    cp target/$TARGET/release/csrf-cookie-token $stage/

    cd $stage
    tar czf "$src/${CRATE_NAME}-${TARGET_NAME}.tar.gz" *
    cd $src

    rm -rf $stage
}

main