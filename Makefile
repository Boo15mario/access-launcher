.PHONY: all build install srpm

all: build

build:
	cargo build --release

install: build
	install -Dm755 target/release/access-launcher /usr/bin/access-launcher
	install -Dm644 access-launcher.desktop /usr/share/applications/access-launcher.desktop
	install -Dm644 access-launcher.svg /usr/share/icons/hicolor/scalable/apps/access-launcher.svg

SPEC := access-launcher.spec
NAME := $(shell rpmspec -q --qf '%{name}' $(SPEC))
VERSION := $(shell rpmspec -q --qf '%{version}' $(SPEC))
TARBALL := $(NAME)-$(VERSION).tar.gz

srpm:
	git archive --format=tar.gz --prefix=$(NAME)-$(VERSION)/ -o $(TARBALL) HEAD
	rpmbuild -bs --define "_sourcedir $(PWD)" --define "_srcrpmdir $(PWD)" --define "_specdir $(PWD)" $(SPEC)
