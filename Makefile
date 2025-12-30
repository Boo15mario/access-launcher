.PHONY: srpm

SPEC := access-launcher.spec
NAME := $(shell rpmspec -q --qf '%{name}' $(SPEC))
VERSION := $(shell rpmspec -q --qf '%{version}' $(SPEC))
TARBALL := $(NAME)-$(VERSION).tar.gz

srpm:
	git archive --format=tar.gz --prefix=$(NAME)-$(VERSION)/ -o $(TARBALL) HEAD
	rpmbuild -bs --define "_sourcedir $(PWD)" --define "_srcrpmdir $(PWD)" --define "_specdir $(PWD)" $(SPEC)
