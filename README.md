# Clippy

🚧 Project currently under construction. Please pe patient!

Clippy is a simple lightweight and easy to use clipboard history manager for
[Wayland](https://wayland.freedesktop.org/) based systems. I started this project with intentions of making a rust port of [cliphist](https://github.com/sentriz/cliphist) but I decided to implement my own set of ideas such as not needing to use a decoding step to recall a clip.

## Features
- persistent clipboard history saved to disk
- easy to interface with pickers (*ie:* **dmenu**, **rofi**, **anyrun**, **fzf**)
- preserves clips byte by byte. With the exception of:
    - leading/trailing whitespace/newline characters
- man page docs and shell completion for (**bash**, **zsh**, **fish**)

### Requirements
- [wl-clipboard](https://github.com/bugaevc/wl-clipboard) Implements the core copy and paste functionality (this project will not work without it)

## Installation
- Using your distro's package manager. Refer to the list below
- [releases page](https://github.com/JustinBacher/clippy/releases/latest)

### Packaging
[![](https://repology.org/badge/vertical-allrepos/clippy.svg?columns=4)](https://repology.org/project/clippy/versions)

