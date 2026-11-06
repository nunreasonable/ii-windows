(function (root, factory) {
	"use strict";
	if (typeof module === "object" && module.exports) module.exports = factory;
	else root.createRequestCoordinator = factory;
})(typeof globalThis !== "undefined" ? globalThis : this, function createRequestCoordinator() {
	"use strict";
	const queues = new Map();
	const generations = new Map();

	function nextGeneration(kind, action) {
		const key = `${kind}:${action}`;
		const generation = (generations.get(key) || 0) + 1;
		generations.set(key, generation);
		return generation;
	}

	function isCurrent(kind, action, generation) {
		return generations.get(`${kind}:${action}`) === generation;
	}

	function enqueue(action, operation) {
		const previous = queues.get(action) || Promise.resolve();
		const result = previous.then(operation, operation);
		queues.set(action, result.then(() => undefined, () => undefined));
		return result;
	}

	return { nextGeneration, isCurrent, enqueue };
});
