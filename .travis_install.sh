#!/bin/bash

git clone https://github.com/zeromq/zeromq4-1
cd zeromq4-1
git checkout "tags/v4.1.4"
./autogen.sh
./configure --with-libsodium=no
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
