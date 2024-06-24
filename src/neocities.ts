import {NeocitiesError} from "./error"
import {tryFetch, tryResponseJson} from "./misc"
import {err, ok, type Result} from "./result"

export interface Host {
	protocol: "https" | "http"
	name: string
}

export enum LoginResult {
	OK = "ok",
	INVALID_AUTH = "invalid-auth",
	NETWORK_ERROR = "network-error",
}

export enum UploadResult {
	OK = "ok",
	INVALID_FILE_TYPE = "invalid-file-type",
	INVALID_AUTH = "invalid-auth",
	NETWORK_ERROR = "network-error",
}

export enum DeleteResult {
	OK = "ok",
	MISSING_FILES = "missing-files",
	INVALID_AUTH = "invalid-auth",
	NETWORK_ERROR = "network-error",
}

export enum ListError {
	INVALID_AUTH = "invalid-auth",
	NETWORK_ERROR = "network-error",
}

export interface FileEntry {
	is_directory: false
	path: string
	size: number
	updated_at: string
	sha1_hash: string
}

export interface DirectoryEntry {
	is_directory: true
	path: string
	updated_at: string
}

export class Neocities {
	apiKey?: string

	constructor(private host: Host = {protocol: "https", name: "neocities.org/api"}) {}

	private _(): string {
		return `${this.host.protocol}://${this.host.name}`
	}

	private ensureLoggedIn() {
		if (!this.apiKey) {
			throw new NeocitiesError("Not logged in, use neocities.login() first.")
		}
	}

	async login(username: string, password: string): Promise<LoginResult> {
		const response_ = await tryFetch(`${this._()}/key`, {
			headers: {Authorization: `Basic ${btoa(`${username}:${password}`)}`},
		})
		if (!response_.ok) return LoginResult.NETWORK_ERROR
		const response = response_.value
		const data_ = await tryResponseJson(response)
		if (!data_.ok) return LoginResult.NETWORK_ERROR
		const data = data_.value
		if (data.error_type === "invalid_auth") return LoginResult.INVALID_AUTH
		if (data.result === "error") {
			throw new NeocitiesError(`${data.error_type}: ${data.message}`)
		}
		this.apiKey = data.api_key
		return LoginResult.OK
	}

	async upload(files: File[]): Promise<UploadResult> {
		if (files.length === 0) return UploadResult.OK
		this.ensureLoggedIn()
		const form = new FormData()
		for (const file of files) {
			form.append(file.name, file)
		}
		const response_ = await tryFetch(`${this._()}/upload`, {
			method: "POST",
			headers: {Authorization: `Bearer ${this.apiKey}`},
			body: form,
		})
		if (!response_.ok) return UploadResult.NETWORK_ERROR
		const response = response_.value
		const data_ = await tryResponseJson(response)
		if (!data_.ok) return UploadResult.NETWORK_ERROR
		const data = data_.value
		if (data.error_type === "invalid_auth") return UploadResult.INVALID_AUTH
		if (data.result === "invalid_file_type") return UploadResult.INVALID_FILE_TYPE
		if (data.result === "error") {
			throw new NeocitiesError(`${data.error_type}: ${data.message}`)
		}
		return UploadResult.OK
	}

	async delete(files: string[]): Promise<DeleteResult> {
		if (files.length === 0) return DeleteResult.MISSING_FILES
		this.ensureLoggedIn()
		const form = new FormData()
		for (const file of files) {
			form.append("filenames[]", file)
		}
		const response_ = await tryFetch(`${this._()}/delete`, {
			method: "POST",
			headers: {Authorization: `Bearer ${this.apiKey}`},
			body: form,
		})
		if (!response_.ok) return DeleteResult.NETWORK_ERROR
		const response = response_.value
		const data_ = await tryResponseJson(response)
		if (!data_.ok) return DeleteResult.NETWORK_ERROR
		const data = data_.value
		if (data.error_type === "invalid_auth") return DeleteResult.INVALID_AUTH
		if (data.result === "missing_files") return DeleteResult.MISSING_FILES
		if (data.result === "error") {
			throw new NeocitiesError(`${data.error_type}: ${data.message}`)
		}
		return DeleteResult.OK
	}

	async list(): Promise<Result<(FileEntry | DirectoryEntry)[], ListError>> {
		this.ensureLoggedIn()
		const response_ = await tryFetch(`${this._()}/list`, {
			headers: {Authorization: `Bearer ${this.apiKey}`},
		})
		if (!response_.ok) return err(ListError.NETWORK_ERROR)
		const response = response_.value
		const data_ = await tryResponseJson(response)
		if (!data_.ok) return err(ListError.NETWORK_ERROR)
		const data = data_.value
		if (data.error_type === "invalid_auth") return err(ListError.INVALID_AUTH)
		if (data.result === "error") {
			throw new NeocitiesError(`${data.error_type}: ${data.message}`)
		}
		return ok(data.files)
	}
}
