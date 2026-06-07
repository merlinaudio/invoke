export class Flight {
	modules: Record<string, Record<string, any>>
	constructor(modules: Record<string, Record<string, any>>) {
		this.modules = modules
	}
}
