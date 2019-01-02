set -euxo pipefail

main() {
    cargo check --target $TARGET

    if [ $TRAVIS_RUST_VERSION = nightly ]; then
        cargo check --target $TARGET --features inline-asm
    fi

    case $TARGET in
        arm*v7r-none-eabi*)
            ;;

        *)
            cargo test --target $TARGET

            ./check-blobs.sh
            ;;
    esac
}

main
