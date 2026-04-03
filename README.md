# Wallpaper Engine Linux

`wallpaper-engine-gui` is a Tauri desktop app that makes Wallpaper Engine wallpapers more usable on Linux with a cleaner setup flow, a live Steam Workshop browser, and local runtime management.

The current implemented desktop backend is KDE Plasma via [`catsout/wallpaper-engine-kde-plugin`](https://github.com/catsout/wallpaper-engine-kde-plugin). This pass also introduces a backend layer so the app can grow toward other non-GNOME targets later without tangling the whole codebase together.

## What Works Now

- Live Steam Workshop browsing instead of placeholder wallpaper cards
- Search-driven workshop results from inside the app
- Managed SteamCMD installation in the app runtime
- One-click KDE plugin install flow from the app
- Local install and apply history
- Direct workshop ID install/apply flow
- Packaging scaffolding for AppImage, DEB, RPM, and Arch

## Desktop Support

- KDE Plasma is the only fully implemented backend right now
- The codebase now has a desktop backend layer so future support can be added without rewriting the app flow
- GNOME is intentionally not part of this pass

## Project Layout

- App backend: [`src-tauri`](/home/don/dev/wallpaper-engine-gui/src-tauri)
- Bundled frontend: [`src-tauri/src/index.html`](/home/don/dev/wallpaper-engine-gui/src-tauri/src/index.html)
- Packaging assets: [`packaging`](/home/don/dev/wallpaper-engine-gui/packaging)

## Development

If you already have the system libraries available:

```bash
cd src-tauri
cargo tauri dev
```

If you are using Nix, the repo already includes [`shell.nix`](/home/don/dev/wallpaper-engine-gui/shell.nix) with the GTK/WebKit dependencies needed for local Tauri builds:

```bash
nix-shell --run 'cd src-tauri && cargo check'
nix-shell --run 'cd src-tauri && cargo tauri dev'
```

The actively maintained desktop app lives under [`src-tauri`](/home/don/dev/wallpaper-engine-gui/src-tauri). Running Cargo commands from that directory avoids pulling in older top-level prototype code that is no longer the primary runtime target.

## Notes

- KDE plugin installation expects `git` and `cmake` on the host system.
- Live workshop browsing and SteamCMD installation both need network access at runtime.
- The live workshop browser relies on parsing public Steam workshop pages, so Steam markup changes may require parser updates later.

## Packaging

See [`packaging/README.md`](/home/don/dev/wallpaper-engine-gui/packaging/README.md) for the current packaging targets and starter files.

## Next Good Targets

- Improve workshop parser resilience and result quality
- Add richer wallpaper metadata caching
- Add import/export or pinning for install history
- Add another non-GNOME desktop backend when its runtime path is ready
