export class NeocitiesError extends Error {
	constructor(message: string) {
		super(message)
		this.name = "NeocitiesError"
	}
}
