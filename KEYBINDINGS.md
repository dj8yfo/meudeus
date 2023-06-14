# Config

When configuring keys only `Ctrl` and `Alt` combinations are allowed with single char.
E.g., `"ctrl-a"` and `"alt-x"` are valid string values for keymap config, and `"ctrl-alt-t"` and `"ctrl-ty"` are not.

# Keybindings of `explore` command

- `explore` mode

 | Binding| Congirurable | Effect                                                                                                      |
 |--------|--------------|-------------------------------------------------------------------------------------------------------------|
 | Ctrl-c |  no          | Abort                                                                                                       |
 | ESC    |  no          | Abort                                                                                                       |
 | Enter  |  no          | Open selected note in editor                                                                                |
 |        |              |                                                                                                             |
 | Ctrl-o |  yes         | Open selected note with `xdg-open` or `dio`, or its corresponding counterpart in the host operating system  |
 | Ctrl-h |  yes         |  Populate skim selection with backlinks of selected note                                                    |
 | Ctrl-l |  yes         |  Populate skim selection with forward links of selected note                                                |
 | Ctrl-t |  yes         |  Toggle preview type of notes                                                                               |
 | Ctrl-w |  yes         |  Widen skim selection to full list of all notes                                                             |
 | Ctrl-s |  yes         |  Switch mode to `surf` with the selected note as the root of surfed subtree                                 |
 | Ctrl-k |  yes         |  Switch mode to `checkmark` for task items of selected note                                                 |
 | Alt-r  |  yes         |  Rename selected note                                                                                       |
 | Alt-l  |  yes         |  Create a link from selected note to another, selected in next skim iteration                               |
 | Alt-u  |  yes         |  Remove a link from selected note to one of its forward links                                               |
 | Alt-d  |  yes         |  Remove selected note                                                                                       |
 | Alt-c  |  yes         |  Create a new note/tag, which will become one of selected note's forward links                              |
 | Alt-f  |  yes         |  Toggle/invert the direction of links. Backlinks become forward links                                       |
 | Alt-s  |  yes         |  Splice note: populate selection list with its children, reachable by forward links                         |
 | Alt-n  |  yes         |  Narrow selection to single or multiple selected notes                                                      |
 | Alt-o  |  yes         |  Decrease threshold of nested level for unlisted inner items (links, task items)                            |
 | Alt-p  |  yes         |  Increase threshold of nested level for unlisted inner items (links, task items)                            |
 | Alt-a  |  yes         |  Push selected note to `GLOBAL` stack                                                                       |
 | Ctrl-a |  yes         |  Switch mode to `stack` (viewing `GLOBAL` stack)                                                            |

- `surf` mode

 | Binding  | Congirurable  | Effect                                                                                                    |
 |----------|---------------|-----------------------------------------------------------------------------------------------------------|
 | Ctrl-c   |  no           | Abort                                                                                                     |
 | ESC      |  no           | Abort                                                                                                     |
 | Enter    |  no           | Open selected `[markdown link]()` with a command, depending on [markdown link]()'s type                   |
 |          |               |                                                                                                           |
 | Ctrl-o   |  yes          | Open selected link with `xdg-open` or `dio`, or its corresponding counterpart in the host operating system|
 | Ctrl-j   |  yes          |  Jump to selected `[markdown link]()`'s position in editor                                                |
 | Ctrl-e   |  yes          |  Return to `explore` mode (in `explore` command) or abort `surf` command                                  |

- `checkmark` mode

 | Binding          | Congirurable | Effect                                                                            |
 |------------------|--------------|-----------------------------------------------------------------------------------|
 | Ctrl-c           |  no          | Abort                                                                             |
 | ESC              |  no          | Abort                                                                             |
 | TAB (skim)       |  no          | Select and move down                                                              |
 | Shift+TAB (skim) |  no          | Select and move up                                                                |
 | Enter            |  no          | Toggle state todo/done of multiple selected task items                            |
 |                  |              |                                                                                   |
 | Ctrl-j           |  yes         |  Jump to selected task item's position in editor                                  |
 | Ctrl-y           |  yes         |  Copy selected task item's subtree to clipboard                                   |
 | Ctrl-w           |  yes         |  Widen context of task items to all tasks, parse again from file                  |
 | Ctrl-l           |  yes         |  Narrown context of task items to subtree of selected task item                   |
 | Ctrl-e           |  yes         |  Return to `explore` mode (in `explore` command) or abort `checkmark` command     |

- `stack` mode

 | Binding | Configurable | Effect                                                                                                                                     |
 |---------|--------------|--------------------------------------------------------------------------------------------------------------------------------------------|
 | Ctrl-c  | no           | Abort                                                                                                                                      |
 | ESC     | no           | Abort                                                                                                                                      |
 | Enter   | no           | If called from withing `explore` command switch mode back to `explore` with selected note. From `stack` command print note's name and exit.|
 |         |              |                                                                                                                                            |
 | Ctrl-t  | yes          |  Toggle preview type of notes                                                                                                              |
 | Alt-p   | yes          |  Pop note from `GLOBAL` stack                                                                                                              |
 | Alt-t   | yes          |  Move note to top of `GLOBAL` stack                                                                                                        |
 | Ctrl-e  | yes          |  Return to `explore` mode without changes, as if `stack` mode wasn't switched to (if called from withing `explore` command)                |
 | Alt-u   | yes          |  Move note one up; it will stay selected on next select iteration; it must be deselected explicitly                                        |
 | Alt-d   | yes          |  Move note one down; it will stay selected on next select iteration; it must be deselected explicitly                                      |
 | Ctrl-q  | yes          |  Deselect all notes; may be used after selecting multiple notes with `TAB` or after moving one note up & down                              |

- common more and less obvious keybindings from vanilla skim

 | Binding        | Effect                                                                                                                                                            |
 |----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------|
 | Ctrl-p, Ctrl-k | Move up by one in skim selection                                                                                                                                  |
 | Ctrl-n, Ctrl-j | Move down by one in skim selection, note that Ctrl-j is taken by jump action in `surf` and `checkmark` in default cfg, but that action can be bound to other keys |
 | PageUp         | Move up by many items in skim selection                                                                                                                           |
 | PageDown       |Move down by many items in skim selection                                                                                                                          |
 | Ctrl-r         |Switch matching mode to regex and back to fuzzy?                                                                                                                   |
 | Shift-ArrowUp  | Scroll preview port up (without mouse)                                                                                                                            |
 | Shirt-ArrowDown| Scroll preview port down (without mouse)                                                                                                                          |

