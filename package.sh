#!/bin/sh
# Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
# top-level directory of this distribution and at
# https://intecture.io/COPYRIGHT.
#
# Licensed under the Mozilla Public License 2.0 <LICENSE or
# https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
# modified, or distributed except according to those terms.

# Undefined vars are errors
set -u

# Globals
prefix=""
libdir=""
libext=""
ostype="$(uname -s)"
make="make"

case "$ostype" in
    Linux)
        prefix="/usr"
        libext="so"

        # When we can statically link successfully, we should be able
        # to produce vendor-agnostic packages.
        if [ -f "/etc/redhat-release" ]; then
            ostype="redhat"
            libdir="$prefix/lib64"
        elif [ -f "/etc/debian_version" ]; then
            ostype="debian"
            libdir="$prefix/lib"
        else
            echo "unsupported Linux flavour" >&2
            exit 1
        fi
        ;;

    FreeBSD)
        ostype="freebsd"
        prefix="/usr/local"
		libdir="$prefix/lib"
        libext="so"
        make="gmake"
        ;;

    Darwin)
        ostype="darwin"
        prefix="/usr/local"
		libdir="$prefix/lib"
        libext="dylib"
        ;;

    *)
        echo "unrecognized OS type: $ostype" >&2
        exit 1
        ;;
esac

main() {
    local _cargodir=$(pwd)
    local _tmpdir="$(mktemp -d 2>/dev/null || mktemp -d -t intecture)"
    cd "$_tmpdir"

    # ZeroMQ dependency
    if ! $(pkg-config --exists libzmq) || [ $(pkg-config libzmq --modversion) != "4.2.0" ]; then
        curl -sSOL https://github.com/zeromq/libzmq/releases/download/v4.2.0/zeromq-4.2.0.tar.gz
        tar zxf zeromq-4.2.0.tar.gz
        cd zeromq-4.2.0
        ./autogen.sh
        ./configure --prefix=$prefix --libdir=$libdir
        $make
        $make install
        cd ..
    fi

    # CZMQ dependency
    if ! $(pkg-config --exists libczmq) || [ $(pkg-config libczmq --modversion) != "4.0.1" ]; then
        curl -sSOL https://github.com/zeromq/czmq/releases/download/v4.0.1/czmq-4.0.1.tar.gz
        tar zxf czmq-4.0.1.tar.gz
        cd czmq-4.0.1
        ./configure --prefix=$prefix --libdir=$libdir
        $make
        $make install
        cd ..
    fi

    # Build and install project assets
    cargo build --release --manifest-path "$_cargodir/Cargo.toml"
    cp "$_cargodir/target/release/libinapi.$libext" "$libdir"
    cp "$_cargodir/bindings/c/inapi.h" "$prefix/include/"

    cd "$_cargodir/bindings/php5"
    phpize
    ./configure --prefix=$prefix --libdir=$libdir
    $make
    $make install

    cd ../php7
    phpize
    ./configure --prefix=$prefix --libdir=$libdir
    $make
    $make install
    cd "$_tmpdir"

    local _version=$(grep -m1 -E '^version\s?=\s?\"[0-9.]+\"' "$_cargodir/Cargo.toml" | awk '{split($3, a, "\""); print a[2]}')
    local _pkgdir="inapi-$_version"

    # Create package dir structure
    mkdir "$_pkgdir"
    mkdir "$_pkgdir/include"
    mkdir "$_pkgdir/lib"
    mkdir "$_pkgdir/lib/pkgconfig"

    # Project assets
    cp "$_cargodir/target/release/libinapi.$libext" "$_pkgdir/libinapi.$libext.$_version"
    cp "$_cargodir/bindings/c/inapi.h" "$_pkgdir/inapi.h"
    cp "$_cargodir/bindings/php5/.libs/inapi.so" "$_pkgdir/inapi.so.5"
    cp "$_cargodir/bindings/php7/.libs/inapi.so" "$_pkgdir/inapi.so.7"

    # ZeroMQ assets
    cp "$libdir/libzmq.so.5.1.0" "$_pkgdir/lib/"
    cp "$libdir/pkgconfig/libzmq.pc" "$_pkgdir/lib/pkgconfig/"
    cp "$prefix/include/zmq.h" "$_pkgdir/include/"

    # CZMQ assets
    cp "$libdir/libczmq.so.4.0.0" "$_pkgdir/lib/"
    cp "$libdir/pkgconfig/libczmq.pc" "$_pkgdir/lib/pkgconfig/"
    cp "$prefix/include/czmq.h" "$_pkgdir/include/"
    cp "$prefix/include/czmq_library.h" "$_pkgdir/include/"
    cp "$prefix/include/czmq_prelude.h" "$_pkgdir/include/"
    cp "$prefix/include/zactor.h" "$_pkgdir/include/"
    cp "$prefix/include/zarmour.h" "$_pkgdir/include/"
    cp "$prefix/include/zauth.h" "$_pkgdir/include/"
    cp "$prefix/include/zbeacon.h" "$_pkgdir/include/"
    cp "$prefix/include/zcert.h" "$_pkgdir/include/"
    cp "$prefix/include/zcertstore.h" "$_pkgdir/include/"
    cp "$prefix/include/zchunk.h" "$_pkgdir/include/"
    cp "$prefix/include/zclock.h" "$_pkgdir/include/"
    cp "$prefix/include/zconfig.h" "$_pkgdir/include/"
    cp "$prefix/include/zdigest.h" "$_pkgdir/include/"
    cp "$prefix/include/zdir.h" "$_pkgdir/include/"
    cp "$prefix/include/zdir_patch.h" "$_pkgdir/include/"
    cp "$prefix/include/zfile.h" "$_pkgdir/include/"
    cp "$prefix/include/zframe.h" "$_pkgdir/include/"
    cp "$prefix/include/zgossip.h" "$_pkgdir/include/"
    cp "$prefix/include/zhash.h" "$_pkgdir/include/"
    cp "$prefix/include/zhashx.h" "$_pkgdir/include/"
    cp "$prefix/include/ziflist.h" "$_pkgdir/include/"
    cp "$prefix/include/zlist.h" "$_pkgdir/include/"
    cp "$prefix/include/zlistx.h" "$_pkgdir/include/"
    cp "$prefix/include/zloop.h" "$_pkgdir/include/"
    cp "$prefix/include/zmonitor.h" "$_pkgdir/include/"
    cp "$prefix/include/zmsg.h" "$_pkgdir/include/"
    cp "$prefix/include/zpoller.h" "$_pkgdir/include/"
    cp "$prefix/include/zproxy.h" "$_pkgdir/include/"
    cp "$prefix/include/zrex.h" "$_pkgdir/include/"
    cp "$prefix/include/zsock.h" "$_pkgdir/include/"
    cp "$prefix/include/zstr.h" "$_pkgdir/include/"
    cp "$prefix/include/zsys.h" "$_pkgdir/include/"
    cp "$prefix/include/zuuid.h" "$_pkgdir/include/"

    # Configure installer.sh paths
    sed "s~{{prefix}}~$prefix~" < "$_cargodir/installer.sh" |
    sed "s~{{libdir}}~$libdir~" |
    sed "s~{{libext}}~$libext~" |
    sed "s~{{version}}~$_version~" > "$_pkgdir/installer.sh"
    chmod u+x "$_pkgdir/installer.sh"

    local _pkgstoredir="$_cargodir/.pkg/$ostype"
    mkdir -p "$_pkgstoredir"

    local _tarball="$_pkgstoredir/$_pkgdir.tar.bz2"
    tar -cjf "$_tarball" "$_pkgdir"

    cd "$_cargodir"
}

main || exit 1
