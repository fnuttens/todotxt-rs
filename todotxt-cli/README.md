# `todotxt-cli`

CLI crate. Powered by [clap](https://github.com/clap-rs/clap).

```console
$ todotxt-cli help
Ask more of your todo.txt file

Usage: todotxt-cli[EXE] [COMMAND]

Commands:
  add      Add a new task to todo.txt
  do       Mark a task as done
  rm       Remove a task
  archive  Move all completed tasks to done.txt
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

```console
$ todotxt-cli help add
Add a new task to todo.txt

Usage: todotxt-cli[EXE] add [OPTIONS] <todo>

Arguments:
  <todo>  Description of your task

Options:
  -p, --priority <A-Z>              Priority of your task
  -c, --creation-date <YYYY-MM-DD>  Creation date for this task
      --no-creation-date            Disable creation date for this task
  -h, --help                        Print help

```

```console
$ todotxt-cli help archive
Move all completed tasks to done.txt

Usage: todotxt-cli[EXE] archive

Options:
  -h, --help  Print help

```

```console
$ todotxt-cli help do
Mark a task as done

Usage: todotxt-cli[EXE] do <task-id>

Arguments:
  <task-id>  Identifying number for the accomplished task

Options:
  -h, --help  Print help

```

```console
$ todotxt-cli help rm
Remove a task

Usage: todotxt-cli[EXE] rm <task-id>

Arguments:
  <task-id>  Identifying number for the task to remove

Options:
  -h, --help  Print help

```
