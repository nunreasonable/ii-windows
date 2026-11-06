"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const createRequestCoordinator = require("./request-coordinator.js");

function deferred() {
	let resolve, reject;
	const promise = new Promise((res, rej) => { resolve = res; reject = rej; });
	return { promise, resolve, reject };
}

function flush() {
	return new Promise((resolve) => setImmediate(resolve));
}

function sourceLoader(coordinator, state) {
	return async (action, promise) => {
		const generation = coordinator.nextGeneration("source", action);
		state[action] = { loading: true };
		try {
			const value = await coordinator.enqueue(action, () => promise);
			if (coordinator.isCurrent("source", action, generation)) state[action] = { value };
		} catch (error) {
			if (coordinator.isCurrent("source", action, generation)) state[action] = { error };
		}
	};
}

test("an old source response cannot replace the latest request state", async () => {
	const coordinator = createRequestCoordinator();
	const state = {};
	const load = sourceLoader(coordinator, state);
	const oldRequest = deferred();
	const latestRequest = deferred();
	const oldLoad = load("install", oldRequest.promise);
	const latestLoad = load("install", latestRequest.promise);

	oldRequest.resolve("preview");
	await oldLoad;
	assert.deepEqual(state.install, { loading: true });

	latestRequest.resolve("stable");
	await latestLoad;
	assert.deepEqual(state.install, { value: "stable" });
});

test("an old source error cannot replace the latest request state", async () => {
	const coordinator = createRequestCoordinator();
	const state = {};
	const load = sourceLoader(coordinator, state);
	const oldRequest = deferred();
	const latestRequest = deferred();
	const oldLoad = load("install", oldRequest.promise);
	const latestLoad = load("install", latestRequest.promise);

	oldRequest.reject(new Error("old failure"));
	await oldLoad;
	assert.deepEqual(state.install, { loading: true });

	latestRequest.resolve("stable");
	await latestLoad;
	assert.deepEqual(state.install, { value: "stable" });
});

test("source requests for one action run in FIFO order after failures", async () => {
	const coordinator = createRequestCoordinator();
	const calls = [];
	const first = deferred();
	const firstRequest = coordinator.enqueue("install", async () => {
		calls.push("first:start");
		await first.promise;
		calls.push("first:end");
		throw new Error("first failure");
	});
	const secondRequest = coordinator.enqueue("install", async () => {
		calls.push("second:start");
		return "stable";
	});

	await flush();
	assert.deepEqual(calls, ["first:start"]);
	first.resolve();
	await assert.rejects(firstRequest, /first failure/);
	assert.equal(await secondRequest, "stable");
	assert.deepEqual(calls, ["first:start", "first:end", "second:start"]);
});

test("source queues for different actions are independent", async () => {
	const coordinator = createRequestCoordinator();
	const install = deferred();
	let installFinished = false;
	let updateFinished = false;
	const installRequest = coordinator.enqueue("install", async () => {
		await install.promise;
		installFinished = true;
	});
	const updateRequest = coordinator.enqueue("update", async () => {
		updateFinished = true;
	});

	await updateRequest;
	assert.equal(updateFinished, true);
	assert.equal(installFinished, false);
	install.resolve();
	await installRequest;
	assert.equal(installFinished, true);
});

test("an old preflight response cannot replace the latest request state", async () => {
	const coordinator = createRequestCoordinator();
	const state = {};
	const oldRequest = deferred();
	const latestRequest = deferred();
	async function load(promise) {
		const generation = coordinator.nextGeneration("preflight", "install");
		state.install = { loading: true };
		const value = await promise;
		if (coordinator.isCurrent("preflight", "install", generation)) state.install = { value };
	}
	const oldLoad = load(oldRequest.promise);
	const latestLoad = load(latestRequest.promise);

	latestRequest.resolve("latest");
	await latestLoad;
	oldRequest.resolve("old");
	await oldLoad;
	assert.deepEqual(state.install, { value: "latest" });
});
