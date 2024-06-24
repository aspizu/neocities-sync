# neocities-sync

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

Bun is required, install it from <https://bun.sh/>

Just run:

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

Run your build command. Given that your build output is in the `dist` directory,
we can use `neocities-sync` to upload the files to neocities.

```bash
neocities-sync --username foo --password bar --path dist --ignore-disallowed-file-types --state .state
```

Here, we use `--state .state` to store the state outside the `dist` directory because
it will be emptied on every build.

Now, every time you make changes to your app, re-run the build command and then
run `neocities-sync`. It will only upload the files that have been modified.

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
