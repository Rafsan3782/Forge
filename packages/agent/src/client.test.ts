import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { BridgeClient } from "./client.ts";

describe("BridgeClient unit tests", () => {
  test("initializes with disconnected state", () => {
    const client = new BridgeClient({
      port: 9999,
      token: "test_token_12345",
    });

    assert.equal(client.getState(), "disconnected");
    assert.equal(client.isConnected(), false);
  });

  test("state listener receives initial and updated states", () => {
    const client = new BridgeClient({
      port: 9999,
      token: "test_token_12345",
    });

    const states: string[] = [];
    const unsubscribe = client.onStateChange((state) => {
      states.push(state);
    });

    assert.deepEqual(states, ["disconnected"]);
    client.disconnect();
    assert.equal(client.getState(), "disconnected");
    unsubscribe();
  });
});
