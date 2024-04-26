[![Tests](https://github.com/Phydon/gerf/actions/workflows/rust.yml/badge.svg)](https://github.com/Phydon/gerf/actions/workflows/rust.yml)


# gerf

***Ge**nerates **R**andom **F**ile with a specified size and random (or not so random) file content*


#### Todo

- generate different "random" content
  - generate content with only numbers
  - generate content with only words
  - generate alphanumeric content
  - ...


## Examples

- generate a file with the default name 'gerf.txt' with a size of 100 Bytes

```shell
gerf 100     
```

- generate a file with a custom name 'wasd.md' and with a size of 12 MB

```shell
gerf 12 --mb --path wasd.md    
```


## Usage

### Short Usage

```
Usage: gerf [OPTIONS] [SIZE] [COMMAND]

Commands:
  log, -L, --log  Show content of the log file
  help            Print this message or the help of the given subcommand(s)

Arguments:
  [SIZE]  The size the generated file should have

Options:
  -e, --exceed       Exceed the default maximum filesize
      --gb           Treat size input as gigabyte [Gb]
      --kb           Treat size input as kilobyte [Kb]
      --mb           Treat size input as megabyte [Mb]
  -o, --override     Override an existing file
  -p, --path <PATH>  Set a custom filepath / filename [default: gerf.txt]
  -h, --help         Print help (see more with '--help')
  -V, --version      Print version
```

### Long Usage

```
Usage: gerf [OPTIONS] [SIZE] [COMMAND]

Commands:
  log, -L, --log  Show content of the log file
  help            Print this message or the help of the given subcommand(s)

Arguments:
  [SIZE]
          The size the generated file should have
          Default unit is [Bytes]

Options:
  -e, --exceed
          Exceed the default maximum filesize
          DANGER: Can produce a very large file

      --gb
          Treat size input as gigabyte [Gb]
          Not as bytes [b]

      --kb
          Treat size input as kilobyte [Kb]
          Not as bytes [b]

      --mb
          Treat size input as megabyte [Mb]
          Not as bytes [b]

  -o, --override
          Override an existing file

  -p, --path <PATH>
          Set a custom filepath / filename

          [default: gerf.txt]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Installation

### Windows

via Cargo or get the ![binary](https://github.com/Phydon/gerf/releases)

