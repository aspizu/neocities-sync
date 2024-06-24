# neocities-sync

Sync files to neocities while minimizing the number of API requests.

[Vite]: https://vitejs.dev/
[Astro]: https://astro.build/
[Next.js]: https://nextjs.org/


You can use this to deploy any project which builds as a static site. For example, you can deploy [Vite][], [Astro][], or [Next.js][] projects to neocities.

[supporter]: https://neocities.org/supporter
[file types]: https://neocities.org/site_files/allowed_types

`neocities-sync` will:

- Only upload files that have been modified.
- Delete files which exist on neocities, but don't exist locally.
- Store SHA1 hashes locally inside a `.state` file.
- If the `.state` file doesn't exist, it will fetch all files from neocities and store them in the `.state` file.
- If `--ignore-disallowed-file-types` is set, it will ignore disallowed [file types][]. Use this if you are NOT a paid [supporter][].

## Installation

Install Bun <https://bun.sh/>

No need to install, just run

```bash
bunx github:aspizu/neocities-sync
```

## Usage

```
Usage: neocities-sync [options]

Sync files to neocities while minimizing the number of API requests.

Options:
  -V, --version                   output the version number
  --username <USERNAME>           Neocities username.
  --password <PASSWORD>           Neocities password.
  --path <PATH>                   Path to sync.
  --state <STATE>                 Path to state file. (default: <PATH>/.state)
  --ignore-disallowed-file-types  Ignore disallowed file types. (default: false)
  -h, --help                      display help for command
```

### Deploy a Vite/Astro/Next.js app to neocities

```bash
neocities-sync --username foo --password bar --path dist --ignore-disallowed-file-types --state .state
```

Here, we use `--state .state` to store the state outside the `dist` directory because
it will be emptied on every build.

## User Interface

The shell script `neocities-sync.sh` provides a basic user interface using
zenity.

```bash
./neocities-sync.sh
```

![Username and password dialog.](/assets/screenshot.png)

## Development

```bash
git clone https://github.com/aspizu/neocities-sync
cd neocities-sync
bun run src/index.ts
```

## Contributing

Pull requests are welcome.
