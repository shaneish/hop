# hop

## commands
1) `hop`
  * cd into a named directory
2) `hop add`
  * add a new named directory to `~/..config/hop`
  * detect whether is a file or a folder and parse appropriately
3) `hop run`
  * execute a file
  * map file types to execution commands
4) `hop edit`
  * open a file to edit
  * map file types to editors
5) `hop ls`
  * list out available hops
6) `hop brb`
  * temporarily mark a directory to easily jump back to

## implementation
1) 

## potential tools 
* [symlink](https://crates.io/crates/symlink)
  * Create new symlinks
* [toml](https://docs.rs/toml/latest/toml/)
  * Parse conf files
* [pathdiff](https://docs.rs/pathdiff/0.1.0/pathdiff/)
  * Get relative path between one directory and another file
