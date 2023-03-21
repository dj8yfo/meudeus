![crates.io](https://img.shields.io/crates/v/mds.svg)

# Concentrate by means of relaxation

![Alt](./logo.jpeg "Concentrate by means of relaxation")

# Flow

1. Install the binary itself.

```
cargo install --locked mds 
```

2. Install external commands used in default config, besides `firefox`, which is the default browser for opening links, 
by running [install_dependencies.sh](./install_dependencies.sh)
```
wget -O - https://raw.githubusercontent.com/dj8yfo/mds/master/install_dependencies.sh  | bash
```
3. Create config at `$HOME/.config/mds/config.kdl` with [content](./config.kdl).
  - Edit the folder, where you'd like to put notes on your system. (Replace `/home/user/notes` default value)
4. Check your config got correctly fetched up.
  ```
  mds debug-cfg
  ```
  - if you see `no command specified` message after updating `<0.5.0` -> `0.5.0`, 
  that most likely means that config needs extra field `pipe-$SNIPPET_TEXT-into` (see default config). 

5. Initialize .sqlite database in your notes folder with
  ```
  mds init  
  ```

6. Enjoy ([fast gif tutorial is here](./tutorial.gif)):

  Both `explore` and `surf` are equipped with `<Ctrl-h>` and `<Ctrl-l>` keybindings to follow 
  backlinks and forward links respectively.

  Both `explore` and `surf` are equipped with `<Ctrl-t>` keyding to toggle between details and structural
  preview of current note or note subgraph respectively. This renders `p/print` command somewhat redundant.

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
    help       Print this message or the help of the given subcommand(s)

  Options:
    -c, --color    whether color output should be forced
    -h, --help     Print help
    -V, --version  Print version
  ```

