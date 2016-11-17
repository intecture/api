TARGET = release
PREFIX = /usr/local
LIBEXT = so

all: remote

local:
ifeq ($(TARGET), release)
	cargo build --release --no-default-features --features=local-run
else
	cargo build --no-default-features --features=local-run
endif

remote:
ifeq ($(TARGET), release)
	cargo build --release
else
	cargo build
endif

c: install
	install -m 0644 bindings/c/inapi.h $(PREFIX)/include/

php5: c
	cd bindings/php5
	phpize
	./configure
	make
	make install
	cd ../..

php7: c
	cd bindings/php7
	phpize
	./configure
	make
	make install
	cd ../..

install:
	install -m 0644 target/$(TARGET)/libinapi.$(LIBEXT) $(PREFIX)/lib/

uninstall:
	rm -f $(PREFIX)/lib/libinapi.$(LIBEXT)

test-local:
ifeq ($(TARGET), release)
	cargo test --release --no-default-features --features=local-run
else
	cargo test --no-default-features --features=local-run
endif

test-remote:
ifeq ($(TARGET), release)
	cargo test --release
else
	cargo test
endif

clean:
	cargo clean
