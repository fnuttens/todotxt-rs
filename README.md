# todotxt-rs

> Ask more of your [todo.txt](http://todotxt.org/) file.

⚠️ This software is currently under development, it is not ready for everyday usage (yet)

## What it is about

This tool aims at giving a practical front-end to the _todo.txt_ format.

It is cut into two parts:
- a library crate responsible for the logic of interacting with the todo file
- a binary crate that exposes a CLI

Maybe more front-ends will come in the future, such as a desktop GUI, a webapp, or even a mobile app!

## Roadmap

- Commands
    - [x] add task
    - [ ] mark task as done
    - [ ] remove task
    - [ ] list tasks
    - [ ] prioritize task
    - [ ] deprioritize task
    - [ ] archive tasks
    - [ ] update task
- [ ] manage a serialized configuration

## Built using these great crates

- [chrono](https://github.com/chronotope/chrono)
- [clap](https://github.com/clap-rs/clap)
- [colored](https://github.com/mackwic/colored)
