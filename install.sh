#!/bin/bash
set -e

echo "============================================="
echo "  Installing OpenBN: Bangla Phonetic IME     "
echo "============================================="

# 1. Compile the workspace in release mode
echo "--> Compiling OpenBN in release mode..."
cargo build --release

# 2. Deploy binary to ~/.local/bin
echo "--> Creating local bin directory..."
mkdir -p "$HOME/.local/bin"

echo "--> Copying binary..."
killall openbn-daemon >/dev/null 2>&1 || pkill -f openbn-daemon >/dev/null 2>&1 || true
cp target/release/openbn-daemon "$HOME/.local/bin/openbn-daemon"
chmod +x "$HOME/.local/bin/openbn-daemon"

# 3. Generate and deploy IBus component XML
echo "--> Generating openbn.xml with home directory: $HOME"
sed "s|__HOME__|${HOME}|g" component/openbn.xml.template > /tmp/openbn.xml

echo "--> Installing component XML to /usr/share/ibus/component/openbn.xml (requires sudo)..."
sudo cp /tmp/openbn.xml /usr/share/ibus/component/openbn.xml
sudo chmod 644 /usr/share/ibus/component/openbn.xml
rm /tmp/openbn.xml

# 4. Restart IBus daemon to apply changes
echo "--> Restarting IBus daemon..."
if command -v ibus >/dev/null 2>&1; then
    ibus restart
    echo "--> IBus restarted successfully!"
else
    echo "WARNING: 'ibus' command not found in your PATH."
    echo "If you are running Fedora/Ubuntu, IBus is typically active."
    echo "Please ensure the IBus daemon is restarted to load the engine."
fi

echo "============================================="
echo "  OpenBN Installation Complete!              "
echo "============================================="
echo ""
echo "To activate OpenBN on your system:"
echo "1. Open GNOME Settings (or your desktop settings)."
echo "2. Navigate to 'Keyboard' -> 'Input Sources'."
echo "3. Click '+' (Add Input Source)."
echo "4. Select 'Bengali' -> 'Bengali (OpenBN)'."
echo "5. To toggle between English and Bangla typing modes:"
echo "   Use the shortcut 'Ctrl + Space' when OpenBN is active."
echo "============================================="
