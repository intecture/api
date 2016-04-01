#!/bin/bash

curl -sSOL https://download.libsodium.org/libsodium/releases/libsodium-1.0.8.tar.gz
curl -sSOL https://download.libsodium.org/libsodium/releases/libsodium-1.0.8.tar.gz.sig
curl -sSOL https://download.libsodium.org/jedi.gpg.asc
gpg --import jedi.gpg.asc
gpg --verify libsodium-1.0.8.tar.gz.sig libsodium-1.0.8.tar.gz
tar zxf libsodium-1.0.8.tar.gz
cd libsodium-1.0.8
./configure
make
sudo make install
cd ..

curl -sSOL https://github.com/zeromq/zeromq4-1/archive/v4.1.4.tar.gz
tar zxf v4.1.4.tar.gz
cd zeromq4-1-4.1.4
./autogen.sh
./configure --with-libsodium
make -j 8
sudo make install
cd ..

git clone https://github.com/zeromq/czmq
cd czmq
./autogen.sh
./configure
make
sudo make install
cd ..
