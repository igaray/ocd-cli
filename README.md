# OCD

A CLI file management swiss army knife.

**ocd** is a quirky collection of simple command-line tools for manipulating
files, their names, and paths.

It currently has two subcommands: a mass renamer and a file sorter.

### License and Credits

### Requirements

### Installation

### Usage

## Mass ReNamer

It operates on single files or groups of files by populating a buffer with a listing of files, parsing the arguments to generate a sequence of actions, processing the actions in order and applying their effects to the contents of the file name buffer. The final state for each file name is shown and confirmation is requested before renaming the files.

## Time Stamp Sorter

The time stamp sorter will examine all files in a directory and check them
against a regular expression to see whether they contain something that looks
like a date.

The regular expression is
`"\D*(20[01]\d).?(0[1-9]|1[012]).?(0[1-9]|[12]\d|30|31)\D*"`
which essentially looks for `YYYY?MM?DD` or `YYYYMMDD`, where `YYYY` in
`[2000-2019]`, `MM` in `[01-12]`, and `DD` in `[01-31]`.

If the filename does contain a date it will create a directory named after the
date and move the file into it.
