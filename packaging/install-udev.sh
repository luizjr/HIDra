#!/bin/sh
# Instala a regra udev que dá ao seu usuário acesso aos dois dispositivos.
# Uso:  sudo sh packaging/install-udev.sh   (--uninstall para remover)
set -e
RULE=70-hidra.rules
DEST=/etc/udev/rules.d/$RULE
SRC="$(dirname "$0")/$RULE"

if [ "$1" = "--uninstall" ]; then
    rm -f "$DEST"
    echo "removido $DEST"
else
    if [ "$(id -u)" -ne 0 ]; then
        echo "rode com sudo." >&2
        exit 1
    fi
    install -m 0644 "$SRC" "$DEST"
    echo "instalado $DEST"
fi

udevadm control --reload-rules
udevadm trigger --subsystem-match=hidraw --action=change || true
echo "pronto — reconecte o mouse/teclado (ou o dongle) para aplicar."
