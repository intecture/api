UNAME_S := $(shell uname -s)
TARGET = release
 
ifeq ($(UNAME_S), Linux)
	FEDORA := $(grep -qs Fedora /etc/redhat-release)
	ifeq ($$?, 0)
		USRPATH = /usr/local
		export PATH := $(USRPATH)/bin:$(PATH)
	else
		USRPATH = /usr
	endif
else ifeq ($(UNAME_S), Darwin)
	USRPATH = /usr/local
endif

all:
ifeq ($(TARGET), release)
	$(USRPATH)/bin/cargo build --release
else
	$(USRPATH)/bin/cargo build
endif

install:
	install -m 0644 target/$(TARGET)/libinapi.so $(USRPATH)/lib/

uninstall:
	rm -f $(USRPATH)/lib/libinapi.so

clean:
	$(USRPATH)/bin/cargo clean