# Help

  

  ```
  mds -h
  ```

  ```
  meudeus v0.18.4
  a skim shredder for plain-text papers

  Usage: mds [OPTIONS] <COMMAND>

  Commands:
    debug-cfg  print Debug representtion of config
    init       `initialize` .sqlite database in notes dir, specified by config
    note       create a note [aliases: n]
    tag        create a tag (note without file body) [aliases: t]
    select     select note S, i.e. print its name to stdout
    link       link 2 notes A -> B, selected twice in skim interface [aliases: l]
    unlink     unlink 2 notes A -> B, selected twice in skim interface [aliases: ul]
    remove     remove note R, selected in skim interface [aliases: rm]
    rename     rename note R, selected in skim interface [aliases: mv]
    print      print subgraph of notes and links reachable downwards from selected note P [aliases: p]
    explore    explore notes by <c-h> (backlinks) , <c-l> (links forward) 
                   [aliases: ex]
    surf       surf through all links and code snippets found downwards from selected note S
                   [aliases: s]
    stack      browse GLOBAL stack of notes [aliases: st]
    checkmark  checkmark, toggle state TODO/DONE of multiple task items, found in a selected note C
                   [aliases: k]
    help       Print this message or the help of the given subcommand(s)

  Options:
    -c, --color    whether color output should be forced
    -h, --help     Print help
    -V, --version  Print version
  ```

# Overview

## General

1. Any note can be linked to any number of other notes via a directed `->` link. 
2. Note names are rendered as markdown in skim picker/preview.
3. A note having only a name, but devoid of earthly file body is also considered a note, but is called a tag instead.
4. Keybindings of most of secondary actions can be reconfigured in [config](./config.kdl). 
5. A command can go through 1 or more modes during its dialogue, e.g. 
  - `surf` command starts in `explore` mode.
  - after a user chooses a note, whose subtree he/she wants to explore for urls/code snippets, `surf` command switches to `surf` mode.
  - `surf` command stays in a `surf` mode loop for the note initially selected after a primary action with `Enter` or some secondary action has been selected for one of found urls/code snippets.

## Explore mode

1. All of `explore`, `surf` and `checkmark` commands start in `explore` mode.
2. `explore` command can switch to `surf` or `checkmark` mode and then back to `explore` mode. 
3. `explore` command includes the functionality of most of other commands (`link`, `unlink` , `rename`, `delete`, `surf`, `checkmark`, etc), and is used as the only entrypoint for program's interface by its author.
4. In `explore` mode ` Ctrl-h ` (backlinks) and ` Ctrl-l ` (forwardlinks) bindings are available.
5. ` Ctrl-t ` keybinding may be used to toggle 
  between **structural links** -> **structural task** -> **details** -> **(cycle)** preview of current note or 
  note subgraph respectively. This rendered `p/print` command somewhat redundant. 

## Surf mode

1. `surf` command/mode may be used for searching for all `[description](url/file_path/dir_path)` markdown links and `'''code_block'''` found downwards from a note S, selected for `surf`.
2. Destination in `[description](destination)` markdown links is matched against `world.surf-parsing.url-regex` regex in [config](./config.kdl). 
  - If it matches, it's considered a url link. 
  - Otherwise, it's considered local filesystem link, either absolute or relative (no `file://` protocol prefix required).   
  - If `filesystem_link:37` matches `world.surf-parsing.file-dest-has-line-regex` regex in [config](./config.kdl) it's considered a `$FILE:$LINE` link. 
  - Local filesystem link has any env variables replaced with their values, e.g. `$HOME/path/to/file` gets expanded to `/home/user/path/to/file`.
3. `'''code_block'''` description is parsed as the first line of `'''code_block'''`, comments `# bash comment` or `// C comment` may be used for informative descriptions.
4. Syntax in `'''code_block'''`can be hinted for highlight in preview by specifying tag \`\`\`syntax_tag, e.g. \`\`\`bash or \`\`\`javascript.

## Checkmark mode

1. `checkmark` command/mode may be used to parse out trees of `- [ ] description` task items and allows navigating/toggling them into `- [x] description` state.

## Stack mode 

1. `stack` mode is a simple way to manage priorities of notes. 
2. A note can be pushed to stack by *Alt-a* from `explore` mode of `explore` command. 
3. A switch to `stack` mode from `explore` mode of `explore` command can be made by *Ctrl-a*.
4. By selecting a note with `Enter` in `stack` mode one returns to `explore` mode with the note selected.
5. In `stack` mode a note can be popped off stack with *Alt-p*.
6. Selected note can be moved to top of stack by *Alt-t*.
7. Currently only single `GLOBAL` stack is supported. It may be extended to multiple stacks in a future.

# [Keybindings](./KEYBINDINGS.md)
