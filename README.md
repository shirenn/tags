# Tags

Tags is an utility to edit audio file from the commandline based on taglib. It
was greatly inspired by [taggie](https://github.com/ravicious/taggie) another
rust based tag editor. It aims to provide multiple ways to get and set tags.

To view tags you can simply run :
```bash
$ tags view [-j] file1 file2
```
With -j giving the output in a nice json format to make it easier to work with
other commandline utility.

To edit the tags, you can choose from three different ways :
  - the quick way :
  ```bash
  $ tags quickedit --artist Ratatat file1 file2
  ```
  - with an external editor :
  ```bash
  $ tags editor file1 file2
  ```
  - from a json formatted entry:
  ```bash
  $ echo "{ "file1": { "album": "LP3" }, "file2": { "album": "Magnifique" }}" | tags edit
  ```

## Installation
To install tags, you'll need `taglib` and `cargo`. As `tags` is not yet featured
in crates, you will need to clone the sources locally for now.
```bash
$ git clone https://github.com/shirenn/tags
$ cargo install --path tags
```
