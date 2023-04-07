# `todotxt-cli`

CLI crate. Powered by [clap](https://github.com/clap-rs/clap).

```console
$ todotxt-cli help
todotxt-rs [VERSION]
Florent Nuttens
Ask more of your todo.txt file

USAGE:
    todotxt-cli[EXE] [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add        Add a new task to todo.txt
    archive    Move all completed tasks to done.txt
    do         Mark a task as done
    help       Print this message or the help of the given subcommand(s)
    rm         Remove a task

```

```console
$ todotxt-cli help add
todotxt-cli[EXE]-add 
Add a new task to todo.txt

USAGE:
    todotxt-cli.exe add [OPTIONS] <todo>

ARGS:
    <todo>    Description of your task

OPTIONS:
    -c, --creation-date <YYYY-MM-DD>    Creation date for this task
    -h, --help                          Print help information
        --no-creation-date              Disable creation date for this task
    -p, --priority <A-Z>                Priority of your task

```

```console
$ todotxt-cli help archive
todotxt-cli[EXE]-archive 
Move all completed tasks to done.txt

USAGE:
    todotxt-cli.exe archive

OPTIONS:
    -h, --help    Print help information

```

```console
$ todotxt-cli help do
todotxt-cli[EXE]-do 
Mark a task as done

USAGE:
    todotxt-cli.exe do <task-id>

ARGS:
    <task-id>    Identifying number for the accomplished task

OPTIONS:
    -h, --help    Print help information

```

```console
$ todotxt-cli help rm
todotxt-cli[EXE]-rm 
Remove a task

USAGE:
    todotxt-cli.exe rm <task-id>

ARGS:
    <task-id>    Identifying number for the task to remove

OPTIONS:
    -h, --help    Print help information

```
