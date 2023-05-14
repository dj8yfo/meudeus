![crates.io](https://img.shields.io/crates/v/mds.svg)

# Sample screencast

[![asciicast](https://asciinema.org/a/QtWI1lMVbQ52LMP7Yf6u7QvAA.svg)](https://asciinema.org/a/QtWI1lMVbQ52LMP7Yf6u7QvAA)

# [Installation](./INSTALLATION.md)

# About

`mds` is a cli tool for
1. navigating a collection of markdown notes
2. creating new notes and linking them together. Notes' metadata and inter-note links are stored outside of them in .sqlite database.
3. opening `[description](links)` found inside of markdown notes
4. jumping to these `[description](links)`' location in markdown in editor (if one needs to change them)
5. etc.

It links to external tools, such as `bat` via [config](./config.kdl). 

`mds` works with any *dumb* editor.  It doesn't require editor to have any kind of rich plugin system.

# [Usage](./USAGE.md)



# Colors 

- Some color themes for markdown elements in `world.color.theme` field of [config](./config.kdl) can be found at [rainglow/sublime](https://github.com/rainglow/sublime)
and previewed at this awesome website [rainglow.io](https://rainglow.io/preview/).
- if the patchwork in markdown irritates you, please remember, that `settings.background` value in a theme 
is editable
- `world.color.elements` field of [config](./config.kdl) specifies colors of most of other displayed objects.

# [Changelog](./CHANGELOG.md)
