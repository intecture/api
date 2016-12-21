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
prefix="{{prefix}}"
libdir="{{libdir}}"
libext="{{libext}}"
pkgconf="{{pkgconf}}"
pkgconfdir="{{pkgconfdir}}"
version="{{version}}"
os="{{os}}"

do_install_c() {
    need_cmd $pkgconf

    local _one=
    local _two=

    if ! $($pkgconf --exists libzmq); then
        if [ "$os" = "darwin" ]; then
            _one="5"
            _two=$libext
        else
            _one=$libext
            _two="5"
        fi
        install -m 755 lib/libzmq.$libext $libdir/libzmq.$_one.$_two
        ln -s $libdir/libzmq.$_one.$_two $libdir/libzmq.$libext
        install -m 644 lib/pkgconfig/libzmq.pc $pkgconfdir
        install -m 644 include/zmq.h $prefix/include/

        if [ "$os" = "freebsd" ]; then
            install -m 644 lib/libstdc++.so.6 $libdir/
        fi
    fi

    if ! $($pkgconf --exists libczmq); then
        if [ "$os" = "darwin" ]; then
            _one="4"
            _two=$libext
        else
            _one=$libext
            _two="4"
        fi
        install -m 755 lib/libczmq.$libext $libdir/libczmq.$_one.$_two
        ln -s $libdir/libczmq.$_one.$_two $libdir/libczmq.$libext
        install -m 644 lib/pkgconfig/libczmq.pc $pkgconfdir
        install -m 644 include/czmq.h $prefix/include/
        install -m 644 include/czmq_library.h $prefix/include/
        install -m 644 include/czmq_prelude.h $prefix/include/
        install -m 644 include/zactor.h $prefix/include/
        install -m 644 include/zarmour.h $prefix/include/
        install -m 644 include/zauth.h $prefix/include/
        install -m 644 include/zbeacon.h $prefix/include/
        install -m 644 include/zcert.h $prefix/include/
        install -m 644 include/zcertstore.h $prefix/include/
        install -m 644 include/zchunk.h $prefix/include/
        install -m 644 include/zclock.h $prefix/include/
        install -m 644 include/zconfig.h $prefix/include/
        install -m 644 include/zdigest.h $prefix/include/
        install -m 644 include/zdir.h $prefix/include/
        install -m 644 include/zdir_patch.h $prefix/include/
        install -m 644 include/zfile.h $prefix/include/
        install -m 644 include/zframe.h $prefix/include/
        install -m 644 include/zgossip.h $prefix/include/
        install -m 644 include/zhash.h $prefix/include/
        install -m 644 include/zhashx.h $prefix/include/
        install -m 644 include/ziflist.h $prefix/include/
        install -m 644 include/zlist.h $prefix/include/
        install -m 644 include/zlistx.h $prefix/include/
        install -m 644 include/zloop.h $prefix/include/
        install -m 644 include/zmonitor.h $prefix/include/
        install -m 644 include/zmsg.h $prefix/include/
        install -m 644 include/zpoller.h $prefix/include/
        install -m 644 include/zproxy.h $prefix/include/
        install -m 644 include/zrex.h $prefix/include/
        install -m 644 include/zsock.h $prefix/include/
        install -m 644 include/zstr.h $prefix/include/
        install -m 644 include/zsys.h $prefix/include/
        install -m 644 include/zuuid.h $prefix/include/
    fi

    install -m 755 libinapi.$libext.$version $libdir
    if [ ! -e "$libdir/libinapi.$libext" ]; then
        ln -s $libdir/libinapi.$libext.$version $libdir/libinapi.$libext
    fi

    install -m 644 inapi.h $prefix/include/
}

do_install_php() {
    need_cmd php

    local _phpver="$(php --version|grep -E '^PHP')"
    local _major=$(echo $_phpver|awk '{split($2, a, "."); print a[1]}')
    local _minor=$(echo $_phpver|awk '{split($2, a, "."); print a[2]}')
    local _extdir=$(php -r "echo ini_get('extension_dir');")

    if [ $_major -eq 5 ] && [ $_minor -lt 6 ]; then
        echo "Intecture requires PHP version >=5.6. Found version $_major.$_minor." >&2
        exit 1
    fi

    do_install_c

    # The extension dir is not guaranteed to exist
    mkdir -p $_extdir

    case $_major in
        5)
            cp inapi.so.5 $_extdir/inapi.so
            ;;

        7)
            cp inapi.so.7 $_extdir/inapi.so
            ;;

        *)
            echo "Unsupported PHP major version: $1" >&2
            exit 1
            ;;
    esac

    local _phpini=$(php --ini|grep 'Loaded Configuration File'|awk '{print $4}')
    local _additionaldir=$(php --ini|grep 'Scan for additional'|awk '{print $7}')
    if [ -n $_additionaldir ]; then
        echo 'extension=inapi.so' > $_additionaldir/inapi.ini
    elif [ -n $_loadeddir ]; then
        echo 'extension=inapi.so' >> $_phpini
    else
        echo 'Could not find PHP extension ini file' >&2
        exit 1
    fi
}

do_uninstall() {
    local _phpdir=$(php -r "echo ini_get('extension_dir');")
    local _phpini=$(php --ini|grep 'Loaded Configuration File'|awk '{print $4}')
    local _additionaldir=$(php --ini|grep 'Scan for additional'|awk '{print $7}')
    if [ -n $_additionaldir ]; then
        rm -f $_additionaldir/inapi.ini
    elif [ -n $_loadeddir ]; then
        sed 's/extension=inapi.so//' < $_phpini > php.ini.tmp
        mv php.ini.tmp $_phpini
    else
        echo 'Could not find PHP extension ini file' >&2
        exit 1
    fi

    rm -f $libdir/libinapi.$libext \
          $prefix/include/inapi.h \
          $_phpdir/inapi.so
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        echo "need '$1' (command not found)" >&2
        exit 1
    fi
}

main() {
    if [ $# -eq 0 ]; then
        echo "Usage: installer.sh <install|install-c|install-php|uninstall>"
        exit 0
    fi

    case "$1" in
        install)
            do_install_php
            ;;

        install-c)
            do_install_c
            ;;

        install-php)
            do_install_php
            ;;

        uninstall)
            do_uninstall
            ;;

        *)
            echo "Unknown option $1" >&2
            exit 1
            ;;
    esac
}

main "$@"
