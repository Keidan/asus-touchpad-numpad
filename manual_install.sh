#!/bin/bash
set -e

service_user=$(grep '^service_user' assets/service_user.txt | head -1 | sed 's/service_user = "\(.*\)"/\1/')
app_name="asus-touchpad-numpad"
install_dir="/opt/${app_name}"

echo "Creating system user '${service_user}'..."
sudo useradd -r -s /sbin/nologin -G input,i2c,uinput "${service_user}" || true

echo "Installing files to ${install_dir}..."
sudo mkdir -p "${install_dir}"
sudo install -Dm755 -o "${service_user}" -g ${service_user} target/release/${app_name} "${install_dir}/${app_name}"
sudo install -Dm644 -o "${service_user}" -g ${service_user} assets/config.json "${install_dir}/config.json"
sudo install -Dm744 -o "${service_user}" -g ${service_user} assets/uninstall.sh "${install_dir}/uninstall.sh"

echo "Installing systemd service..."
sudo install -Dm644 assets/${app_name}.service "/etc/systemd/system/${app_name}.service"
sudo systemctl daemon-reload
sudo systemctl enable --now "${app_name}"

echo "Installing udev rules..."
sudo install -Dm644 assets/40-asus-touchpad-numpad.rules "/usr/lib/udev/rules.d/40-asus-touchpad-numpad.rules"
sudo udevadm control --reload-rules
sudo udevadm trigger

echo "Installation complete."