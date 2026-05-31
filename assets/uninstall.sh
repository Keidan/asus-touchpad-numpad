#!/bin/bash
set -e

name="asus-touchpad-numpad"
install_dir="/opt/${name}"
service_user="asus-numpad"

echo "Stopping and disabling service..."
sudo systemctl disable --now "${name}" || true
sudo rm -f "/etc/systemd/system/${name}.service"
sudo systemctl daemon-reload

echo "Removing udev rules..."
sudo rm -f /usr/lib/udev/rules.d/40-asus-touchpad-numpad.rules
sudo udevadm control --reload-rules
sudo udevadm trigger

echo "Removing system user '${service_user}'..."
sudo userdel ${service_user} || true

echo "Removing installation directory..."
sudo rm -rf "${install_dir}"

echo "Uninstallation complete."