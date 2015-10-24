#!/bin/bash

git clone https://github.com/zeromq/zeromq4-1
cd zeromq4-1
git checkout "tags/v4.1.3"
./autogen.sh
./configure --with-libsodium=no
make -j 8
sudo make install