# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Miscellaneous Tasks

- Fmt

## [0.19.0] - 2024-03-09

### Features

- Replace `hx` with `$EDITOR` in config

### Miscellaneous Tasks

- Changelog update
- .

## [0.18.6] - 2023-10-05

### Features

- Multiple select delete

### Miscellaneous Tasks

- Fmt + vers update

## [0.18.5] - 2023-06-17

### Documentation

- Add note about env-substitute crate usage

### Features

- Added `select` flag to `explore` command as in `mds explore --select 'snippet | rust' --select syntaxes --select lib`

## [0.18.4] - 2023-06-15

### Documentation

- Update `stack` mode with `deselect_all` keybinding

### Features

- [**breaking**] Add move_up_by_one and move_down_by_one actions
- [**breaking**] Add deselect_all binding to stack mode to clean selection of items
- Return multiple selection from stack, remove unnecessary narrowing of skim selection after some of secondary actions in explore mode

### Miscellaneous Tasks

- Add lint to ci
- Fix or supress clippy lints
- Add prepush check to justfile

## [0.18.3] - 2023-06-08

### Miscellaneous Tasks

- Add test and build to ci

## [0.18.2] - 2023-06-07

### Features

- Keybinding <Ctrl-e> to quit to `explore` mode from `stack` without changes

## [0.18.1] - 2023-06-06

### Features

- Implement miette Reports for config errors

## [0.18.0] - 2023-05-28

### Features

- [**breaking**] Implement config of most of keys

### Miscellaneous Tasks

- Update changelog

## [0.17.2] - 2023-05-15

### Features

- Implement open with `xdg-open` or `dio` via *C-o* binding from `surf` and `explore`; (`opener` crate)

### Miscellaneous Tasks

- .
- Update readme

## [0.17.1] - 2023-04-29

### Features

- Change direction of tree printed on toggling direction of links

### Miscellaneous Tasks

- Tag

## [0.17.0] - 2023-04-29

### Features

- [**breaking**] Parse $FILE:$LINE destinations of [description](destination) links; change in config

### Miscellaneous Tasks

- V0.16.0 change mentioned

## [0.16.0] - 2023-04-24

### Features

- Implement pushing notes to `GLOBAL` stack and switching to viewing it

### Miscellaneous Tasks

- Remove reduntant file
- Replace changelog image with text
- Replace changelog image with text
- Update readme with short description of `stack` mode

## [0.15.4] - 2023-04-20

### Features

- Increase/decrease unlisted items nesting threshold by keybindings

### Miscellaneous Tasks

- Fix doc broken link

## [0.15.3] - 2023-04-19

### Features

- Add grouping tree hint of self.items

## [0.15.2] - 2023-04-18

### Features

- Narrow notes selection list to selected item(s)

## [0.15.1] - 2023-04-18

### Features

- Splice action

## [0.15.0] - 2023-04-18

### Features

- Add action **Alt-f** to toggle direction of links to `explore` command

## [0.14.4] - 2023-04-17

### Features

- Integrate env-substitute crate for [markdow](link) file/dir destinations

### Miscellaneous Tasks

- Tiniest additions to doc

## [0.14.3] - 2023-04-16

### Miscellaneous Tasks

- Small corrections to doc

## [0.14.2] - 2023-04-15

### Miscellaneous Tasks

- Update readme
- Bump published version to propagate readme
- Fix typos and ambiguities in readme

## [0.14.1] - 2023-04-15

### Features

- Extract elements' colors to config

### Miscellaneous Tasks

- Fmt cfg

## [0.14.0] - 2023-04-15

### Features

- `world.color.theme` config field

## [0.13.7] - 2023-04-14

### Bug Fixes

- Alpabetic order of forward links

### Features

- Invoke `surf` and `checkmark` with bindings

### Miscellaneous Tasks

- Fixate cyan for tags in truecolor escapes
- Change skim position upwards
- Fixate red for special tags
- Move checkmark surf preview back to the right

## [0.13.5] - 2023-04-09

### Features

- Delete note/tag from `explore` interface

## [0.13.3] - 2023-04-09

### Features

- Link forward links from `explore` interface

## [0.13.2] - 2023-04-09

### Features

- Improve `markdown` rendering performace

## [0.13.1] - 2023-04-08

### Features

- Rename note from explore by <Alt-r>

### Miscellaneous Tasks

- Pics

## [0.13.0] - 2023-04-08

### Features

- Widen <C-w> binding; markdown syntax highlight for notes' names

## [0.12.3] - 2023-04-08

### Features

- Narrow and widen via <Ctrl-l> and <Ctrl-w> bindings

### Miscellaneous Tasks

- Roady
- Reduce pause after surf action

## [0.12.2] - 2023-04-04

### Features

- Copy selected task_item subtree to clipboard by <C-y>

### Miscellaneous Tasks

- Add hints of current stage to skim prompt

## [0.12.1] - 2023-04-02

### Features

- Jump to selected link/snippet by <C-j> binding

## [0.12.0] - 2023-04-02

### Features

- Jump to selected item by <C-j> binding

## [0.11.6] - 2023-04-01

### Bug Fixes

- Improve links rendering performance in surf command

## [0.11.5] - 2023-04-01

### Bug Fixes

- Add crate for `wl-copy`
- Make skim preview not staggering for tasks
- Make note preview cached
- Remove lazy preview evaluation and caching of preview
- Order of self::tasks and tasks of subnotes
- Improve tasks preview precompute perf a bit

### Features

- Add a command
- Open command
- Make open loop infinetely unless explicitly aborted
- Explore cmd
- Color in fuzzy search
- Surf command
- Help and config
- Unlink command
- Add remove command
- Rename command
- Add code blocks parsing to surf
- Print, select commands and watch_mdtree script
- Make navigable by <c-h> and <c-l> as well
- Structural preview of notes
- Parse out task-items, display as subtrees
- Toggle state of multiple selection

### Miscellaneous Tasks

- Rename package
- Add gif tutorial
- Make Structure the default preview type
- Replace pic of logo
- Make default skim proportion 35/65 (fuzzy list/preview)
- Pics
- Normal names for commands and short aliases redefined
- .

<!-- generated by git-cliff -->
