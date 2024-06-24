#!/usr/bin/env bun
import {Glob, SHA1} from "bun"
import Path from "node:path"
import FS from "node:fs/promises"
import packagejson from "../package.json"
import {
	DeleteResult,
	ListError,
	LoginResult,
	Neocities,
	UploadResult,
} from "./neocities"
import {ok, type Result} from "./result"
import {program} from "commander"

const DISALLOWED_FILE_TYPES = `
apng asc atom avif bin cjs css csv dae eot epub geojson gif glb gltf gpg htm html ico
jpeg jpg js json key kml knowl less manifest map markdown md mf mid midi mjs mtl obj
opml osdx otf pdf pgp pls png py rdf resolveHandle rss sass scss svg text toml ts tsv
ttf txt webapp webmanifest webp woff woff2 xcf xml yaml yml`.split(/\s+/)

async function readStateFile(path: string) {
	let file: string
	try {
		file = await FS.readFile(path, "utf-8")
	} catch {
		return
	}
	const state = new Map<string, string>()
	for (const line of file.split("\n")) {
		const [name, hash] = line.split(":")
		state.set(name, hash)
	}
	return state
}

async function writeStateFile(state: Map<string, string>, path: string) {
	const content = [...state].map(([name, hash]) => `${name}:${hash}`).join("\n")
	await FS.writeFile(path, content)
}

async function fetchState(
	neocities: Neocities,
): Promise<Result<Map<string, string>, ListError>> {
	const listResult = await neocities.list()
	if (!listResult.ok) return listResult
	const state = new Map<string, string>()
	for (const file of listResult.value) {
		if (file.is_directory) continue
		state.set(file.path, file.sha1_hash)
	}
	return ok(state)
}

enum SyncResult {
	OK = "ok",
	OUT_OF_SYNC = "out-of-sync",
	INVALID_FILE_TYPE = "invalid-file-type",
	INVALID_AUTH = "invalid-auth",
	NETWORK_ERROR = "network-error",
}

async function sync(
	neocities: Neocities,
	path: string,
	statePath: string | undefined,
	ignoreDisallowedFileTypes: boolean,
): Promise<SyncResult> {
	if (!statePath) {
		statePath = Path.join(path, ".state")
	}
	let currentState = await readStateFile(statePath)
	if (currentState === undefined) {
		const fetchStateResult = await fetchState(neocities)
		if (!fetchStateResult.ok) {
			if (fetchStateResult.value === ListError.INVALID_AUTH) {
				return SyncResult.INVALID_AUTH
			}
			return SyncResult.NETWORK_ERROR
		}
		currentState = fetchStateResult.value
	}
	const newState: Map<string, string> = new Map()
	const toBeUploaded: File[] = []
	const allFiles = await Array.fromAsync(new Glob("**/*").scan(path))
	await Promise.all(
		allFiles.map(async (subpath) => {
			if (
				subpath === Path.relative(path, statePath) ||
				(ignoreDisallowedFileTypes &&
					DISALLOWED_FILE_TYPES.includes(Path.extname(subpath)))
			) {
				return
			}
			const file = await FS.readFile(Path.join(path, subpath))
			const newHash = new SHA1().update(file).digest("hex")
			const oldHash = currentState.get(subpath)
			if (newHash !== oldHash) {
				toBeUploaded.push(new File([file], subpath))
			}
			newState.set(subpath, newHash)
		}),
	)
	const toBeDeleted = Array.from(currentState.keys()).filter(
		(subpath) => !newState.has(subpath),
	)
	const [uploadResult, deleteResult] = await Promise.all([
		neocities.upload(toBeUploaded),
		neocities.delete(toBeDeleted),
		writeStateFile(newState, statePath),
	])
	if (uploadResult === UploadResult.NETWORK_ERROR) return SyncResult.NETWORK_ERROR
	if (uploadResult === UploadResult.INVALID_AUTH) return SyncResult.INVALID_AUTH
	if (uploadResult === UploadResult.INVALID_FILE_TYPE) {
		return SyncResult.INVALID_FILE_TYPE
	}
	if (deleteResult === DeleteResult.NETWORK_ERROR) return SyncResult.NETWORK_ERROR
	if (deleteResult === DeleteResult.INVALID_AUTH) return SyncResult.INVALID_AUTH
	return SyncResult.OK
}

program
	.name("neocities-sync")
	.version(packagejson.version)
	.description("Sync files to neocities while doing the least amount of API requests.")
	.requiredOption("--username <USERNAME>", "Neocities username.")
	.requiredOption("--password <PASSWORD>", "Neocities password.")
	.requiredOption("--path <PATH>", "Path to sync.")
	.option("--state <STATE>", "Path to state file. (default: <PATH>/.state)")
	.option("--ignore-disallowed-file-types", "Ignore disallowed file types.", false)
	.parse()

const {username, password, path, state, ignoreDisallowedFileTypes} = program.opts()

const neocities = new Neocities()

const loginResult = await neocities.login(username, password)

if (loginResult === LoginResult.INVALID_AUTH) {
	console.error("Username or password is incorrect.")
	process.exit(1)
}

if (loginResult === LoginResult.NETWORK_ERROR) {
	console.error("Network error.")
	process.exit(1)
}

const syncResult = await sync(neocities, path, state, ignoreDisallowedFileTypes)

switch (syncResult) {
	case SyncResult.OK:
		console.log("Synced.")
		break
	case SyncResult.OUT_OF_SYNC:
		console.warn(
			"Out of sync, this happened because your local state file contains file names which do not exist on neocities. To fix this, delete your state file and re-run neocities-sync.",
		)
		break
	case SyncResult.INVALID_FILE_TYPE:
		console.error("Invalid file type, use --ignore-disallowed-file-types to ignore.")
		process.exit(1)
		break
	case SyncResult.INVALID_AUTH:
		console.error("Username or password is incorrect.")
		process.exit(1)
		break
	case SyncResult.NETWORK_ERROR:
		console.error("Network error.")
		process.exit(1)
		break
}
