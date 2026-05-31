#!/bin/bash
set -e

echo "Reading package metadata..."
app_name=$(grep '^name' Cargo.toml | head -1 | sed 's/name = "\(.*\)"/\1/')
app_version=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
service_user=$(grep '^service_user' assets/service_user.txt | head -1 | sed 's/service_user = "\(.*\)"/\1/')
install_dir="/opt/${app_name}"
echo "  name:         ${app_name}"
echo "  version:      ${app_version}"
echo "  service_user: ${service_user}"

echo ""
echo "Generating install scripts..."
cat > /tmp/pre-install.sh << EOF
#!/bin/bash
useradd -r -s /sbin/nologin -G input,i2c,uinput "${service_user}" || true
EOF

cat > /tmp/post-install.sh << EOF
#!/bin/bash
set -e
chown -R "${service_user}:${service_user}" "${install_dir}"
systemctl daemon-reload
systemctl enable --now "${app_name}"
udevadm control --reload-rules
udevadm trigger
EOF

cat > /tmp/pre-uninstall-rpm.sh << EOF
#!/bin/bash

systemctl disable --now "${app_name}" || true
userdel "${service_user}" || true
rm -rf "/opt/${app_name}"
EOF

cat > /tmp/pre-uninstall-deb.sh << EOF
#!/bin/bash

systemctl disable --now "${app_name}" || true
systemctl daemon-reload
rm -f "/etc/systemd/system/${app_name}.service"
rm -f "/usr/lib/udev/rules.d/40-${app_name}.rules"
udevadm control --reload-rules
udevadm trigger
userdel "${service_user}" || true
rm -rf "/opt/${app_name}"
EOF

echo ""
echo "Cleaning up old packages..."
rm -f "${app_name}"*.deb "${app_name}"*.rpm

common_args=(
  --name "${app_name}"
  --version "${app_version}"
  --description "ASUS touchpad numpad daemon"
  --before-install /tmp/pre-install.sh
  --after-install /tmp/post-install.sh
  target/release/${app_name}=${install_dir}/${app_name}
  assets/config.json=${install_dir}/config.json
  assets/uninstall.sh=${install_dir}/uninstall.sh
  assets/${app_name}.service=/etc/systemd/system/${app_name}.service
  assets/40-${app_name}.rules=/usr/lib/udev/rules.d/40-${app_name}.rules
)

echo ""
echo "Building .deb package..."
fpm -s dir -t deb --architecture amd64 --deb-no-default-config-files --before-remove /tmp/pre-uninstall-deb.sh "${common_args[@]}"
echo ".deb package built successfully."

echo ""
echo "Building .rpm package..."
fpm -s dir -t rpm --architecture x86_64 --before-remove /tmp/pre-uninstall-rpm.sh "${common_args[@]}"
echo ".rpm package built successfully."

echo ""
echo "Done. Packages are ready for distribution."