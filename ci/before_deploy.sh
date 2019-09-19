set -ex

cross build --target $TARGET --release
mkdir "${CRATE_NAME}-${TRAVIS_TAG}-${TARGET}"

filename=
case $TRAVIS_OS_NAME in
    linux)
        filename=libredis_hyperminhash.so
        ;;
    osx)
        filename=libredis_hyperminhash.dylib
        ;;
esac

cp target/$TARGET/release/$filename LICENSE README.md "${CRATE_NAME}-${TRAVIS_TAG}-${TARGET}"
tar czf "${CRATE_NAME}-${TRAVIS_TAG}-${TARGET}.tar.gz" "${CRATE_NAME}-${TRAVIS_TAG}-${TARGET}"
