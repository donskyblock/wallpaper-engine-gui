{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Build tools
    pkg-config
    cargo
    rustc

    # Tauri / WebKit dependencies
    glib
    gtk3
    webkitgtk_4_1
    librsvg
    libsoup_3

    # Additional common dependencies
    openssl
    dbus
    libx11
    libxtst

    # For bundling
    squashfsTools
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
    export WEBKIT_DISABLE_COMPOSITING_MODE=1
    export GDK_BACKEND=x11
    '';
}