VERSION=$(shell grep "^[^ ]" Changes | head -1 | cut -f1 -d' ')
RELEASE_BINARY=target/release/safe-rm
DEBUG_BINARY=target/debug/safe-rm
BUILDDIR=safe-rm-$(VERSION)
TARBALL=safe-rm-$(VERSION).tar.gz

all: build

build:
	cargo build
	cargo build --release

dist: $(TARBALL)
	gpg --armor --sign --detach-sig $(TARBALL)

install:
ifneq ($(shell id -u), 0)
	@echo "need superuser to replace rm file"
	exit 1
endif
ifeq ($(wildcard $(RELEASE_BINARY)),)
	@echo "compiled safe-rm file not found"
	exit 1
endif
	mv /bin/rm /bin/real-rm
	mv target/release/safe-rm /bin/rm || true

$(TARBALL):
	mkdir $(BUILDDIR)
	cp -r `cat Manifest` $(BUILDDIR)
	tar zcf $(TARBALL) $(BUILDDIR)
	rm -rf $(BUILDDIR)

clean:
	rm -rf $(TARBALL) $(TARBALL).asc $(BUILDDIR) target

test:
	cargo check --all-targets
	cargo test

lint:
	cargo outdated --root-deps-only
	cargo clippy --quiet
	cargo doc --quiet
	cargo fmt --check

# Tools which aren't in Debian
check:
	cargo-geiger --all-dependencies --quiet true
	cargo audit --deny warnings --quiet
	cargo tarpaulin --fail-under 90
