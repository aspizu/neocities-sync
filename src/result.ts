export type Ok<T> = {ok: true; value: T}
export type Err<E> = {ok: false; value: E}
export type Result<T, E> = Ok<T> | Err<E>

export function err<E>(value: E): Err<E> {
	return {ok: false, value}
}

export function ok<T>(value: T): Ok<T> {
	return {ok: true, value}
}
