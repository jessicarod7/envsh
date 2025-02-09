# envsh

A convenient command-line tool for sending URLs and files to [envs.sh](https://envs.sh)

## Installation

```shell
cargo install envsh
```

## Usage

```shellsession
$ envsh -h
send and shorten stuff with envs.sh

Usage: envsh [OPTIONS] <FILE|URL>
       envsh <COMMAND>

Commands:
  manage  One option lol
  help    Print this message or the help of the given subcommand(s)

Arguments:
  <FILE|URL>  A file or URL to send to the URL host/shortener

Options:
  -d, --display-secret  Print X-Token (and expiry date)
  -s, --shorten         Shorten a URL instead of sending the file it points to
  -S, --secret          Make the resulting URL difficult to guess
  -e, --expires <TIME>  Specify when the URL should expire, in hours or epoch milliseconds
  -h, --help            Print help (see more with '--help')
  
$ envsh manage -h
modify an existing submission

Usage: envsh manage <--expires <EXPIRES>|--delete> <URL> <TOKEN>

Arguments:
  <URL>    Existing envs.sh URL
  <TOKEN>  Secret X-Token to manage URL

Options:
  -e, --expires <EXPIRES>  Specify when the URL should expire, in hours or epoch milliseconds
  -d, --delete             Delete the shared URL immediately (requires `token`)
  -h, --help               Print help

```

### Examples

Upload a local file:

```shellsession
$ envsh test
Succesful! https://envs.sh/Ej-.txt
```

Upload a file at a remote URL:

```shellsession
$ envsh https://example.com/
Succesful! https://envs.sh/tJ.htm
```

Shorten a URL:

```shellsession
$ envsh -s https://example.com/
Succesful! https://envs.sh/20X
```

Upload a local file, expiring after 1 hour, and print X-Token:

```shellsession
$ envsh -e 1 -d .gitignore
Succesful! https://envs.sh/VxK.txt
Expires at 2025-02-09 (Sunday), 14:55:27.476 [America/Toronto]
X-Token: <token>
```

Edit expiry time of uploaded file to 16:00, using a Unix timestamp:

```shellsession
$ envsh manage -e 1739134800000 https://envs.sh/VxK.txt <token>
Change accepted!
```

Delete an uploaded file immediately:

```shellsession
$ envsh manage -d https://envs.sh/VxK.txt <token>
Change accepted!
```