# ClipSnap Makefile
# Automates building and installing ClipSnap on Linux

PREFIX ?= $(HOME)/.local
BINDIR = $(PREFIX)/bin
SYSTEMDUSERDIR = $(HOME)/.config/systemd/user
BINARY = target/release/clipsnap
APP_NAME = clipsnap
SERVICE_FILE = $(APP_NAME).service

.PHONY: all build install uninstall clean

all: build

build:
	cargo build --release

install: build
	@echo "Installing ClipSnap to $(BINDIR)..."
	mkdir -p $(BINDIR)
	install -m 755 $(BINARY) $(BINDIR)/$(APP_NAME)
	
	@echo "Setting up systemd user service..."
	mkdir -p $(SYSTEMDUSERDIR)
	@echo "[Unit]" > $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "Description=ClipSnap Area Screenshot & Clipboard Manager" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "After=graphical-session.target" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "[Service]" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "ExecStart=$(BINDIR)/$(APP_NAME)" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "Restart=always" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "Environment=DISPLAY=:0" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "[Install]" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	@echo "WantedBy=graphical-session.target" >> $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	
	systemctl --user daemon-reload
	systemctl --user enable $(SERVICE_FILE)
	systemctl --user restart $(SERVICE_FILE)
	
	@echo "Installation complete! ClipSnap is now running in the background."
	@echo "Use 'systemctl --user status $(APP_NAME)' to check status."

uninstall:
	@echo "Removing ClipSnap..."
	systemctl --user stop $(SERVICE_FILE) || true
	systemctl --user disable $(SERVICE_FILE) || true
	rm -f $(BINDIR)/$(APP_NAME)
	rm -f $(SYSTEMDUSERDIR)/$(SERVICE_FILE)
	systemctl --user daemon-reload
	@echo "Uninstallation complete."

clean:
	cargo clean
