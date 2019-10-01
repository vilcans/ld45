PROJECT=ld45
VERSION=0.1
FILENAME=$(PROJECT)-$(VERSION)
RELEASE_DIR=release/$(FILENAME)

DEPLOY_PATH=filur:/opt/public/$(PROJECT)

# Windows

.PHONY: release-win
release-win: release/$(FILENAME)-win.zip

release/$(FILENAME)-win.zip: release-dir
	cargo build --release
	cp target/release/$(PROJECT).exe $(RELEASE_DIR)
	(cd release && zip -r $(FILENAME)-win.zip $(FILENAME))

# Linux

.PHONY: release-linux
release-linux: release/$(FILENAME)-linux.tar.gz

release/$(FILENAME)-linux.tar.gz: release-dir
	cargo build --release
	cp target/release/$(PROJECT) $(RELEASE_DIR)
	(cd release && tar czf $(FILENAME)-linux.tar.gz $(FILENAME))

# Common

.PHONY: release-dir
release-dir:
	rm -rf $(RELEASE_DIR)
	mkdir -p $(RELEASE_DIR)
	rsync -a --delete resources/ $(RELEASE_DIR)/resources/
