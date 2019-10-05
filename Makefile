PROJECT=ld45
VERSION=0.2
FILENAME=$(PROJECT)-$(VERSION)
RELEASE_DIR=release/$(FILENAME)

DEPLOY_PATH=filur:/opt/public/$(PROJECT)

ifeq ($(OS), Windows_NT)
	PLATFORM := win
	BLENDER ?= "C:\\Program\ Files\\Blender\ Foundation\\Blender\\blender.exe"
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		PLATFORM := linux
	endif
	ifeq ($(UNAME_S),Darwin)
		PLATFORM := mac
	endif
endif

all: resources

run: resources
	cargo run

release: release-$(PLATFORM)

# Windows

.PHONY: release-win
release-win: release/public/$(FILENAME)-win.zip

release/public/$(FILENAME)-win.zip: resources release-dir
	cargo build --release
	cp target/release/$(PROJECT).exe $(RELEASE_DIR)
	mkdir -p release/public
	(cd release && zip -r public/$(FILENAME)-win.zip $(FILENAME))

# Linux

.PHONY: release-linux
release-linux: release/$(FILENAME)-linux.tar.gz

release/$(FILENAME)-linux.tar.gz: release-dir
	cargo build --release
	ls -l target/release
	cp target/release/$(PROJECT) $(RELEASE_DIR)
	mkdir -p release/public
	(cd release && tar czf public/$(FILENAME)-linux.tar.gz $(FILENAME))

# Mac
.PHONY: release-mac
release-mac: release/$(FILENAME)-mac.zip

release/$(FILENAME)-mac.zip: release-dir
	cargo build --release
	cp -r target/release/$(PROJECT) $(RELEASE_DIR)
	mkdir -p release/public
	(cd release && zip -r public/$(FILENAME)-mac.zip $(FILENAME))

# Common

.PHONY: release-dir
release-dir:
	rm -rf $(RELEASE_DIR)
	mkdir -p $(RELEASE_DIR)
	rsync -a --delete resources/ $(RELEASE_DIR)/resources/

.PHONY: deploy
deploy:
	rsync -avz release/public/* $(DEPLOY_PATH)/

.PHONY: test
test:
	rustfmt --check $$(find src -name '*.rs')
	cargo test

# Resources

.PHONY: resources
resources: gen-resources/mesh.dat

gen-resources:
	mkdir gen-resources

gen-resources/mesh.dat: source-assets/mesh.blend bin/convert_mesh.py
	"$(BLENDER)" $< --background --python bin/convert_mesh.py
