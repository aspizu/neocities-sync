import {err, ok, type Result} from "./result"

export async function tryFetch(
	input: RequestInfo,
	init?: RequestInit,
): Promise<Result<Response, Error>> {
	try {
		return ok(await fetch(input, init))
	} catch (error) {
		if (error instanceof Error) return err(error)
		throw error
	}
}

export async function tryResponseJson(response: Response): Promise<Result<any, Error>> {
	try {
		return ok(await response.json())
	} catch (error) {
		if (error instanceof Error) return err(error)
		throw error
	}
}
