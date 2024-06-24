# neocities-sync

Sync files to neocities while doing the least amount of API requests.

You can use this to deploy a Vite, Astro or Next.js app to neocities.

`neocities-sync` will:

- Only upload files that have been modified.
- Delete files which exist on neocities, but don't exist locally.
- Store SHA1 hashes locally inside a `.state` file.
- If the `.state` file doesn't exist, it will fetch all files from neocities and store them in the `.state` file.
- If `--ignore-disallowed-file-types` is set, it will ignore disallowed file types. Use this if you are NOT a supporter.

## Installation

Install Bun <https://bun.sh/>

No need to install, just run

```bash
bunx github:aspizu/neocities-sync
```

## Usage

```
Usage: neocities-sync [options]

Sync files to neocities while doing the least amount of API requests.

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

## Development

```bash
git clone https://github.com/aspizu/neocities-sync
cd neocities-sync
bun run src/index.ts
```

## Contributing

Pull requests are welcome.
