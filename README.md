# `neocities-sync`

Sync files to [neocities](https://neocities.org/) while doing the least amount of
[API](https://neocities.org/api) requests.

You can use this to deploy a [Vite](https://vitejs.dev/), [Astro](https://astro.build/)
or [Next.js](https://nextjs.org/docs/pages/building-your-application/rendering/static-site-generation)
app to neocities.

`neocities-sync` will:

- Only upload files that have been modified.
- Delete files which exist on neocities, but don't exist locally.
- Store SHA1 hashes locally inside a `.state` file.
- If the `.state` file doesn't exist, it will fetch all file hashes from neocities and store them in the `.state` file.
- If `--ignore-disallowed-file-types` is set, it will ignore [disallowed](https://neocities.org/site_files/allowed_types) file types. Use this if you are _NOT_ a [supporter](https://neocities.org/supporter).

## Installation

```bash
cargo install neocities-sync
```

## Usage

```
❯ neocities-sync --help

|\---/|
| x_x |   neocities-sync
 \_-_/

Sync files to neocities while doing the least amount of API requests.

Usage: neocities-sync <COMMAND>

Commands:
  login   Login to neocities
  logout  Logout from neocities
  sync    Sync a directory to neocities
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
❯ neocities-sync sync --help

Sync a directory to neocities

Usage: neocities-sync sync [OPTIONS] [PATH]

Arguments:
  [PATH]  The directory to sync [default: .]

Options:
  -s, --state <STATE>                 Path to the state file. Used to keep track of the last sync
  -i, --ignore-disallowed-file-types  Use this if you are NOT a supporter
  -h, --help                          Print help
```

### Deploy a Vite/Astro/Next.js app to neocities

Run your build command. Given that your build output is in the `dist` directory,
we can use `neocities-sync` to upload the files to neocities.

First login using the `neocities-sync login` command. It will prompt you for your
username and password (Will not be displayed in the terminal)

```
❯ neocities-sync dist --ignore-disallowed-file-types --state .state
```

Here, we use `--state .state` to store the state outside the `dist` directory because
it will be emptied on every build.

Now, every time you make changes to your app, re-run the build command and then run the
`neocities-sync` command. It will only upload the files that have been modified.

## Contributing

Pull requests are welcome.
