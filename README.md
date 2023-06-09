# lstree
ls and tree combined.
```bash
Usage: lstree [OPTIONS] [File]...

Arguments:
  [File]...  The target file(s) or directories [default: ./]

Options:
  -b, --block-size <Block size>               [default: ]
  -l, --long                                  
  -a, --all                                   
  -d, --dir-only                              
  -r, --reverse                               
  -p, --print-all-file                        
  -e, --exclude <Ignore files base on regex>  [default: ]
  -t, --tree-spacer <Tree spacer>             [default: "  "]
  -h, --help                                  Print help
  -V, --version                               Print version
```
# Examples
Exclude file or directory
```bash
lstree -e target
```
Exclude more complex regex
```bash
lstree -e "(target)|(.rs)"
```
Make file tree grepable. Will print full paths seperated by spaces.
```bash
# Without --print-all-file or -p
lstree -e "target"
# Cargo.toml
# LICENSE
# Cargo.lock
# README.md
# src
#   tree_generator.rs
#   main.rs
#   tree_renderer.rs


# With --print-all-file or -p
lstree -p -e "target"
# Cargo.toml
# LICENSE
# Cargo.lock
# README.md
# src
# src  tree_generator.rs
# src  main.rs
# src  tree_renderer.rs


# So can grep stuff and know the path of the file
lstree -p -e "target" | grep main
# src  main.rs
```
  
  
[CHANGELOG](./CHANGELOG.md)
