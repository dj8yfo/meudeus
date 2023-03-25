![crates.io](https://img.shields.io/crates/v/mds.svg)

# Meta screenshot

![Alt](./logo.jpeg "Concentrate by means of relaxation")

# [Installation](./INSTALLATION.md)

# Enjoy


1. Enjoy ([fast gif tutorial is here](./tutorial.gif)):

  All of `explore`, `surf` and `checkmark` are equipped with `<Ctrl-h>` and `<Ctrl-l>` keybindings to follow 
  backlinks and forward links respectively.

  All of `explore`, `surf` and `checkmark` are equipped with `<Ctrl-t>` keyding to toggle 
  between **details** -> **structural links** -> **structural task** preview of current note or 
  note subgraph respectively. This renders `p/print` command somewhat redundant.

  ```
  mds -h
  ```

  ```
  Usage: mds [OPTIONS] <COMMAND>

  Commands:
    debug-cfg  print Debug representtion of config
    init       `initialize` .sqlite database in notes dir, specified by config
    n          create a note
    t          create a tag (note without file body)
    l          link 2 notes A -> B, selected twice in skim interface
    e          explore notes by <c-h> (backlinks) , <c-l> (links forward)
    s          surf (fuzzy find) through all [markdown reference](links) 
                           and ```code_block(s)```, found in all notes, 
                           reachable by forward links from note/tag S, 
                           selected interactively by skim
    ul         unlink 2 notes A -> B, selected twice in skim interface
    remove     remove note R, selected in skim interface
    rename     rename note R, selected in skim interface
    p          print tree of nodes reachable 
                           by forward links from note P, selected either 
                           non-interactively or in skim interface
    select     select note S, i.e. print it's name to stdout
    chm        checkmark, toggle state TODO/DONE of multiple task items, 
                           found in a selected note
    help       Print this message or the help of the given subcommand(s)

  Options:
    -c, --color    whether color output should be forced
    -h, --help     Print help
    -V, --version  Print version
  ```
# Changelog

![Alt](./changelog.jpeg "Concentrate by means of relaxation")
