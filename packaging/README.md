# Packaging

This project now includes starter packaging assets so the app can move beyond a dev-only Tauri build.

## Targets

- `AppImage`, `deb`, and `rpm` are enabled in [`src-tauri/tauri.conf.json`](/home/don/dev/wallpaper-engine-gui/src-tauri/tauri.conf.json).
- An Arch starter package is included at [`packaging/PKGBUILD`](/home/don/dev/wallpaper-engine-gui/packaging/PKGBUILD).
- A desktop entry is included at [`packaging/wallpaper-engine-linux.desktop`](/home/don/dev/wallpaper-engine-gui/packaging/wallpaper-engine-linux.desktop).

## Tauri Bundles

From the repo root:

```bash
cd src-tauri
cargo tauri build
```

That will generate the enabled Linux bundles when the host has the required GTK/WebKit dependencies available.

## Arch Notes

The included `PKGBUILD` is a starter file, not a published AUR package yet. Before publishing it:

- Replace the source tarball URL with the real release asset or git source.
- Confirm the final binary name if the crate/package name changes.
- Add icon installation once final app branding assets are settled.

## Debian And RPM Notes

Tauri can produce these bundles directly, but you still need a build host with:

- `pkg-config`
- `gtk3`
- `webkit2gtk`
- `libsoup`
- Rust and Cargo
