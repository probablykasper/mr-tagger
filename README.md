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
- opus

![Screenshot](assets/screenshot.png)

## Dev instructions

### Get started

1. Install Node.js
2. Install Rust
3. Follow the [Tauri setup guide](https://tauri.studio/en/docs/getting-started/intro)
4. Run `npm install`

### Useful resources
- https://picard-docs.musicbrainz.org/en/appendices/tag_mapping.html

### Commands
- `npm run dev`: Start app in dev mode
- `npm run build`: Build
- `npm run lint`: Lint
- `npm run format`: Format

### Release new version
1. Update `CHANGELOG.md`
2. Bump the version number in `src-tauri/Cargo.toml`
3. Run `cargo check` to update `Cargo.lock`
4. Create a git tag in the format `v#.#.#`
5. Add release notes to the generated GitHub release and publish it
