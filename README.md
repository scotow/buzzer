[![Release](https://img.shields.io/github/v/tag/scotow/buzzer?label=version)](https://github.com/scotow/buzzer/tags)
[![Build Status](https://img.shields.io/github/actions/workflow/status/scotow/buzzer/docker.yml)](https://github.com/scotow/buzzer/actions)


![Banner](banner.png)

## Features

- Custom room name
- Buzz list
- Buzz selection

## Options

```
Usage: buzzer [OPTIONS]

Options:
  -v, --verbose...         Increase logs verbosity (Error (default), Warn, Info, Debug, Trace)
  -a, --address <ADDRESS>  HTTP listening address [default: 127.0.0.1]
  -p, --port <PORT>        HTTP listening port [default: 8080]
  -h, --help               Print help
  -V, --version            Print version
```

## Docker

```
docker run ghcr.io/scotow/buzzer/buzzer:latest
```

## Screenshots

### Join

![Join](screenshots/1.png)

### Create

![Create](screenshots/2.png)

### Buzz

![Buzz](screenshots/3.png)

### Host

![Host](screenshots/4.png)