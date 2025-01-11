# OCD
A CLI file management swiss army knife.

**ocd** is a quirky collection of simple command-line tools for manipulating
filenames and paths, a file name swiss army knife, if you will. It operates on
single files or groups of files by populating a buffer, parsing the arguments
to generate a sequence of actions, processing the actions in order and applying
their effects to the contents of the file name buffer. The final state for
each file name is shown and confirmation is requested before renaming the files.

### License and Credits
There are many file renaming tools, but this one is mine. I was heavily
inspired by [pyRenamer](https://github.com/SteveRyherd/pyRenamer) by Adolfo
González Blázquez.

Other work similar to this:
* [kevbradwick/pyrenamer](https://github.com/kevbradwick/pyrenamer)
* [italomaia/renamer](https://github.com/italomaia/renamer)
* [Donearm/Renamer](https://github.com/Donearm/Renamer)

### Requirements
Rust must be installed to compile the executable.

### Installation
The Makefile provides `install` and `uninstall` targets which will compile in
release mode and place or remove the resulting binary executable in
`$HOME/.local/bin/`.

### Usage
```bash
$ ocd
A swiss army knife of utilities to work with files.

Usage: ocd <COMMAND>

Commands:
  mrn   Mass Re-Name
  tss   Time Stamp Sort
  id3   Fix ID3 tags
  lphc  Run the Elephant client
  lphs  Start the Elephant server
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## MRN: Mass ReNamer
```bash
Mass Re-Name

Usage: ocd mrn [OPTIONS] <INPUT> [GLOB]

Arguments:
  <INPUT>  The rewrite rules to apply to filenames.
           The value is a comma-separated list of the following rules:
           s                    Sanitize
           cl                   Lower case
           cu                   Upper case
           ct                   Title case
           cs                   Sentence case
           jc                   Join camel case
           jk                   Join kebab case
           js                   Join snaje case
           sc                   Split camel case
           sk                   Split kebab case
           ss                   Split snake case
           r <match> <text>     Replace <match> with <text>
                                <match> and <text> are both single-quote delimited strings
           rdp                  Replace dashes with periods
           rds                  Replace dashes with spaces
           rdu                  Replace dashes with underscores
           rpd                  Replace periods with dashes
           rps                  Replace periods with spaces
           rpu                  Replace periods with underscores
           rsd                  Replace spaces with dashes
           rsp                  Replace spaces with periods
           rsu                  Replace spaces with underscores
           rud                  Replace underscores with dashes
           rup                  Replace underscores with periods
           rus                  Replace underscores with spaces
           i <pos> <text>       Insert <text> at <position>
                                <text> is a single-quote delimited string
                                <pos> may be a non-negative integer or the keyword 'end'
           d <index> <pos>      Delete from <index> to <position>
                                <index> is a non-negative integer,
                                <pos> may be a non-negative integer or the keyword 'end'
           ea <extension>       Change the extension, or add it if the file has none.
           er                   Remove the extension.
           o                    Interactive reorder, see documentation on use.
           p <match> <replace>  Pattern match, see documentation on use.
  [GLOB]   Operate only on files matching the glob pattern, e.g. `-g \"*.mp3\"`.
           If --dir is specified as well it will be concatenated with the glob pattern.
           If --recurse is also specified it will be ignored.

Options:
  -v...                  Sets the verbosity level.
                         Default is low, one medium, two high, three or more debug.
      --silent           Silences all output.
  -d, --dir <DIR>        Run inside a given directory.
                         [default: ./]
      --dry-run          Do not effect any changes on the filesystem.
  -u, --undo             Create undo script.
      --yes              Do not ask for confirmation.
      --git              Rename files by calling `git mv`
  -m, --mode <MODE>      Specified whether the rules are applied to directories,
                         files or all.
                         [default: files]
                         [possible values: all, directories, files]
      --parser <PARSER>  Specifies with parser to use.
                         [default: lalrpop]
                         [possible values: handwritten, lalrpop]
  -r, --recurse          Recurse directories.
  -h, --help             Print help
```

### Verbosity
* Level 0 is silent running and will produce no output.
* Level 1 is low, the default and will only show the final state of the file name buffer.
* Level 2 is medium, and will in addition list the actions to be applied.
* Level 3 is debug level and will in addition show the state of the file name buffer at each step.

### `--yes`
Yes or non-interactive mode will not ask for confirmation and assume the user
confirms everything. Useful for batch scripts.

### `--undo`
Creates a shell script `undo.sh` with commands which may be run to undo the last
renaming operations.

### Rewrite instructions
The `INPUT` argument is a string of comma-separated rewrite instructions.

Example:
```bash
$ ocd mrn "cl,rus,p '{a} {n}' '{2} {1}',i '-FINAL' end"
```

### Pattern Matching

#### Match Pattern

#### Replace Pattern

### Interative Reorder

### Examples

## TSS: Time Stamp Sorter
The time stamp sorter will examine all files in a directory and check them
against a regular expression to see whether they contain something that looks
like a date.

The regular expression combines two common patterns:
- `YYYY?MM?DD` or `YYYYMMDD`,
- `DAY MONTH YEAR` where `DAY` may be 1-31 and `MONTH` is the case-insensitive
  English name of the month or its three-letter abbreviations.

If the filename does contain a date it will create a directory named after the
date and move the file into it.
