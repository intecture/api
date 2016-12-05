TARGET = release
PREFIX = /usr/local
LIBDIR = "$(PREFIX)/lib"

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
	if [ -f target/$(TARGET)/libinapi.dylib ]; then \
		install -m 0644 target/$(TARGET)/libinapi.dylib $(LIBDIR)/; \
	else \
		install -m 0644 target/$(TARGET)/libinapi.so $(LIBDIR)/; \
	fi

uninstall:
	if [ -f $(LIBDIR)/libinapi.dylib ]; then \
		rm -f $(LIBDIR)/libinapi.dylib; \
	else \
		rm -f $(LIBDIR)/libinapi.so; \
	fi

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
