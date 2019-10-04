#!/bin/bash -e

if [ "$TRAVIS_OS_NAME" == "linux" ]; then
    sudo apt-get install libasound2-dev libudev-dev pkg-config
fi

if [ "$TRAVIS_OS_NAME" == 'windows' ]; then
    # Copied from Makefile as Travis doesn't have make installed

    export PROJECT=ld45
    export VERSION=0.2
    export FILENAME=$PROJECT-$VERSION
    export RELEASE_DIR=release/$FILENAME
    rm -rf $RELEASE_DIR
    mkdir -p $RELEASE_DIR
    cp -r resources/ $RELEASE_DIR/

    cargo build --release
    cp target/release/$PROJECT.exe $RELEASE_DIR
    mkdir -p release/public
    (cd release && zip -r public/$(FILENAME)-win.zip $(FILENAME))

else
    # osx and linux
    make release
fi
