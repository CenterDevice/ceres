BASE_NAME=ceres
BIN_NAME=ceres

CARGO_FILE ?= ../../Cargo.toml
BIN_DIR ?= ../../target/release
SRC_DIR ?= ../..
VERSION ?= $(shell sed -n 's/^version.*=.*"\(.*\)"/\1/p' $(CARGO_FILE))
ARCH ?= amd64
DIST ?= bionic

all:
	@echo 'Example: BIN_DIR=../../target/release/ ARCH=amd64 DIST=trusty make package'

package: $(BASE_NAME)-$(VERSION)-$(DIST)-$(ARCH).deb

$(BASE_NAME)-$(VERSION)-$(DIST)-$(ARCH).deb: $(BASE_NAME)/DEBIAN/control $(BASE_NAME)/DEBIAN/conffiles $(BASE_NAME)/DEBIAN/postinst $(BASE_NAME)/DEBIAN/prerm $(BASE_NAME)/usr/bin/$(BIN_NAME) $(BASE_NAME)/usr/share/man/man1/$(BIN_NAME).1.gz $(BASE_NAME)/usr/share/man/man5/$(BIN_NAME).conf.5.gz
	dpkg-deb -b $(BASE_NAME) $@
	dpkg-deb -I $@

clean:
	-rm -fR $(BASE_NAME)
	-rm $(BASE_NAME)-$(VERSION)-$(DIST)-$(ARCH).deb

$(BASE_NAME)/DEBIAN/control: templates/DEBIAN/control $(BASE_NAME)/usr/bin/$(BIN_NAME) $(BASE_NAME)/DEBIAN
	SIZE=`du -s $(BASE_NAME)/usr/bin/$(BIN_NAME) | awk '{ print $$1}'`; \
	SANE_VERSION=`echo $(VERSION) | sed 's/^[a-z]*//'`; \
	MAINTAINER=`cat $(CARGO_FILE) | sed -n 's/^authors.*=.*<\(.*\)>.*/\1/p'`; \
	DESC=`cat $(CARGO_FILE) | sed -n 's/^description.*=.*"\(.*\)"/\1/p'`; \
	sed "s/@@PACKAGE_NAME@@/$(BASE_NAME)/; s/@@VERSION@@/$${SANE_VERSION}/; s/@@ARCH@@/$(ARCH)/; s/@@SIZE@@/$${SIZE}/; s/@@MAINTAINER@@/$${MAINTAINER}/; s/@@DESC@@/$${DESC}/" $< > $@

$(BASE_NAME)/usr/bin/$(BIN_NAME): $(BIN_DIR)/$(BIN_NAME) $(BASE_NAME)/usr/bin
	cp $< $@

$(BASE_NAME)/usr/share/man/man1/$(BASE_NAME).1.gz: $(SRC_DIR)/docs/man1/$(BIN_NAME).1 $(BASE_NAME)/usr/share/man/man1
	cat $< | gzip -c > $@

$(BASE_NAME)/usr/share/man/man5/$(BASE_NAME).conf.5.gz: $(SRC_DIR)/docs/man5/$(BIN_NAME).conf.5 $(BASE_NAME)/usr/share/man/man5
	cat $< | gzip -c > $@

$(BASE_NAME)/DEBIAN/postinst: templates/DEBIAN/postinst
	cp $< $@
	chmod 755 $@

$(BASE_NAME)/DEBIAN/prerm: templates/DEBIAN/prerm
	cp $< $@
	chmod 755 $@

$(BASE_NAME)/DEBIAN/conffiles: templates/DEBIAN/conffiles
	cp $< $@

$(BASE_NAME)/usr/bin:
	mkdir -p $@

$(BASE_NAME)/usr/share/man/man1:
	mkdir -p $@

$(BASE_NAME)/usr/share/man/man5:
	mkdir -p $@

$(BASE_NAME)/DEBIAN:
	mkdir -p $@

