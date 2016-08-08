CARGO := $(shell which cargo)
TARGET = release
USRPATH = /usr/local
LIBEXT = so

all: remote

local:
ifeq ($(TARGET), release)
	$(CARGO) build --release --no-default-features --features=local-run
else
	$(CARGO) build --no-default-features --features=local-run
endif

remote:
ifeq ($(TARGET), release)
	$(CARGO) build --release
else
	$(CARGO) build
endif

install:
	install -m 0644 target/$(TARGET)/libinapi.$(LIBEXT) $(USRPATH)/lib/

uninstall:
	rm -f $(USRPATH)/lib/libinapi.$(LIBEXT)

test-local:
ifeq ($(TARGET), release)
	$(CARGO) test --release --no-default-features --features=local-run
else
	$(CARGO) test --no-default-features --features=local-run
endif

test-remote:
ifeq ($(TARGET), release)
	$(CARGO) test --release
else
	$(CARGO) test
endif

clean:
	$(CARGO) clean
