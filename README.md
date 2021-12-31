<p align="center">
  <img src="./assets/Logo 1024.png" width="80">
</p>
<p align="center">
  <a href="https://github.com/probablykasper/mr-tagger/releases"><b>Download for Mac, Windows or Linux</b></a>
</p>

# Mr Tagger

Music file tagger. For now, it can only edit cover artworks.

Supported file formats:
- aiff
- mp3
- m4a / mp4 / m4p / m4b / m4r / m4v
- wav

![Screenshot](assets/screenshot.png)

## Dev instructions

### Get started

1. Install Node.js
2. Install Rust
3. Follow the [Tauri setup guide](https://tauri.studio/en/docs/getting-started/intro)
4. Run `npm install`

### Commands
- `npm run dev`: Start app in dev mode
- `npm run build`: Build
- `npm run format`: Format
- `npm run check`: Check code

### Release new version
1. Update `CHANGELOG.md`
2. Manually bump the version number in `src-tauri/Cargo.toml`
3. Run `npm run check` to make sure `Cargo.lock` is up to date
4. Commit with a tag in the format `v#.#.#`
5. Add release notes to the generated GitHub release and publish it
