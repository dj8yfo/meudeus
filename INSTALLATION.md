# Install

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
5. Initialize .sqlite database in your notes folder with
  ```
  mds init  
  ```
