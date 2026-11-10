#!/bin/sh
# Install multigit for the current user: the binary into ~/.local/bin, the menu
# entry and the icon into the XDG data directory. Safe to run again.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
app_id=io.github.paxel.multigit
bin_dir="$HOME/.local/bin"
data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"
apps_dir="$data_dir/applications"
icon_dir="$data_dir/icons/hicolor/256x256/apps"

mkdir -p "$bin_dir" "$apps_dir" "$icon_dir"
cp "$here/multigit" "$bin_dir/multigit"
chmod 755 "$bin_dir/multigit"

# A desktop session does not search PATH reliably: point Exec at the binary.
# The path is quoted and escaped as the Desktop Entry spec requires.
escaped=$(printf '%s' "$bin_dir/multigit" | sed -e 's/[\\"`$]/\\\\&/g')
while IFS= read -r line; do
    case "$line" in
        Exec=*) printf 'Exec="%s"\n' "$escaped" ;;
        *) printf '%s\n' "$line" ;;
    esac
done < "$here/$app_id.desktop" > "$apps_dir/$app_id.desktop"

cp "$here/icon_256.png" "$icon_dir/multigit.png"

# Refresh caches where the tools exist; menus also pick the files up on their own.
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$apps_dir" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$data_dir/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 >/dev/null 2>&1 || true
fi

echo "multigit installed to $bin_dir/multigit"
echo "Start it from the application menu, or run: multigit"
