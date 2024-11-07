# 🗳️ Clippy

🚧 Project currently under construction. Please pe patient!

Clippy is a simple lightweight and easy to use clipboard history manager for
[Wayland](https://wayland.freedesktop.org/) based systems. I started this project with intentions of making a rust port of [cliphist](https://github.com/sentriz/cliphist) but I decided to implement my own set of ideas such as not needing to use a decoding step to recall a clip.

## Features
- [x] Persistent clipboard history saved to disk
- [x] Easy to interface with pickers (*ie:* **dmenu**, **rofi**, **anyrun**, **fzf**)
- [x] Preserves clips byte-by-byte.
    <sub>With the exception of leading/trailing whitespace/newline characters</sub>
- [ ] Support for recalling copied images/videos
- [ ] Man page docs
- [ ] Shell completion for (**bash**, **zsh**, **fish**)
- [ ] In memory storage option (non-persistent)

### Requirements
- [wl-clipboard](https://github.com/bugaevc/wl-clipboard) Implements the core copy and paste functionality (this project will not work without it or one that implements the same protocols)

## Installation
- Using your distro's package manager. Refer to the list in [packaging](#packaging)
- Download a binary from the [releases](https://github.com/JustinBacher/clippy/releases/latest) and place in your `$PATH`

### Packaging
[![](https://repology.org/badge/vertical-allrepos/clippy.svg?columns=4)](https://repology.org/project/clippy/versions)

