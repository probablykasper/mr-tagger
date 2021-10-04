# Mr Tagger

<div>
  <img src="assets/Logo%201024.png" align="right" width="80" height="80">
</div>

Music file tagger. For now, it can only edit cover artworks.

Supported file formats
- aiff
- mp3
- m4a / mp4 / m4p / m4b / m4r / m4v
- wav

![Screenshot](assets/screenshot.png)

## Dev instructions

1. Install Node.js (v14 works)
2. Install Rust (v1.54 works)
3. Follow the [Tauri setup guide](https://tauri.studio/en/docs/getting-started/intro)
4. Run `npm install`

### Commands
- `npm run dev`: Start app in dev mode
- `npm run build`: Build
- `npm run lint`: Lint

### Release new version
1. Update `CHANGELOG.md`
2. Manually bump the version number in `src-tauri/Cargo.toml`
3. Check for errors and bump the `Cargo.lock` version number
    ```
    cargo check --manifest-path src-tauri/Cargo.toml
    ```
4. Dispatch the GitHub Release workflow and wait
5. Add release notes to the generated GitHub release and publish it
