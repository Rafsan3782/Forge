import test from "node:test";
import assert from "node:assert/strict";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { z } from "zod";

import {
  GeminiLlmProvider,
  CassettePlayer,
  CassetteRecorder,
  redactKey,
  parseLlmError,
  LlmError,
  loadModelsConfig,
  getRoleConfig,
  estimateCost,
  preparePngInlinePart,
  generateStructuredOutput,
} from "./index.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const sampleCassettePath = path.resolve(__dirname, "fixtures/sample.cassette.json");

test("Models config loads without hardcoded model IDs and estimates token costs", () => {
  const config = loadModelsConfig();
  assert.ok(config.roles.planner);
  assert.ok(config.roles.worker);
  assert.ok(config.roles.fast);
  assert.ok(config.roles.vision);
  assert.ok(config.roles.image);

  assert.equal(typeof config.roles.planner.model, "string");
  assert.ok(config.roles.planner.pricing.input_per_million > 0);
  assert.ok(config.roles.planner.pricing.output_per_million > 0);

  const cost = estimateCost("planner", 1_000_000, 1_000_000);
  assert.ok(cost > 0);
});

test("Key redaction prevents secret exposure in strings and error logs", () => {
  const fakeSecretKey = "AIzaSyFakeSecretKeyForTestingPurposes123";
  const rawLog = `Connection failed with authorization key: ${fakeSecretKey} at endpoint.`;
  const sanitized = redactKey(rawLog, fakeSecretKey);

  assert.ok(!sanitized.includes(fakeSecretKey));
  assert.ok(sanitized.includes("[REDACTED_API_KEY]"));

  const error = parseLlmError(new Error(`Unauthorized: key=${fakeSecretKey}`), fakeSecretKey);
  assert.equal(error.code, "INVALID_KEY");
  assert.ok(!error.message.includes(fakeSecretKey));
});

test("Error classification correctly classifies status codes and quota limits", () => {
  const quotaErr = parseLlmError({ status: 429, message: "Resource exhausted: quota exceeded" });
  assert.equal(quotaErr.code, "QUOTA_EXCEEDED");
  assert.equal(quotaErr.retryable, false);

  const rateLimitErr = parseLlmError({ status: 429, message: "Too many requests per minute" });
  assert.equal(rateLimitErr.code, "RATE_LIMITED");
  assert.equal(rateLimitErr.retryable, true);

  const safetyErr = parseLlmError({ message: "Blocked reason: SAFETY filters tripped" });
  assert.equal(safetyErr.code, "SAFETY_BLOCKED");
  assert.equal(safetyErr.retryable, false);

  const contextErr = parseLlmError({ message: "Input token count exceeds maximum context length" });
  assert.equal(contextErr.code, "CONTEXT_OVERFLOW");
  assert.equal(contextErr.retryable, false);

  const serverErr = parseLlmError({ status: 503, message: "Backend service unavailable" });
  assert.equal(serverErr.code, "SERVER_ERROR");
  assert.equal(serverErr.retryable, true);
});

test("Replay mode deterministically streams turns from cassette fixture", async () => {
  const player = await CassettePlayer.load(sampleCassettePath);
  const provider = new GeminiLlmProvider({
    mode: "replay",
    player,
  });

  const chunks: string[] = [];
  let thoughtReceived = "";
  let toolCallsReceived: unknown[] = [];
  let finishReason = "";

  for await (const chunk of provider.streamTurn(
    [{ role: "user", content: "Create a ground entity with a sprite and box collider." }],
    {
      tools: [
        {
          name: "scene.create_entity",
          description: "create an entity",
          input_schema: { type: "object" },
        },
        {
          name: "scene.set_component",
          description: "set component",
          input_schema: { type: "object" },
        },
      ],
    }
  )) {
    if (chunk.delta) chunks.push(chunk.delta);
    if (chunk.thought) thoughtReceived = chunk.thought;
    if (chunk.tool_calls) toolCallsReceived = chunk.tool_calls;
    if (chunk.finish_reason) finishReason = chunk.finish_reason;
  }

  assert.equal(chunks.join(""), "Creating ground entity...");
  assert.ok(thoughtReceived.includes("I should create the entity first"));
  assert.equal(toolCallsReceived.length, 1);
  assert.equal(finishReason, "tool_calls");
});

test("Structured output helper parses valid schema and recovers on retry", async () => {
  const player = await CassettePlayer.load(sampleCassettePath);
  const provider = new GeminiLlmProvider({
    mode: "replay",
    player,
  });

  const PlayerStatsSchema = z.object({
    health: z.number(),
    speed: z.number(),
    lives: z.number(),
  });

  const result = await generateStructuredOutput(
    provider,
    PlayerStatsSchema,
    [{ role: "user", content: "Generate JSON schema for player stats" }]
  );

  assert.equal(result.health, 100);
  assert.equal(result.speed, 250);
  assert.equal(result.lives, 3);
});

test("Image input converts bytes to PNG inline data part", () => {
  const dummyBytes = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  const part = preparePngInlinePart(dummyBytes);

  assert.equal(part.inlineData.mimeType, "image/png");
  assert.ok(part.inlineData.data.length > 0);
});

test("Abort signal cancels turn stream cleanly", async () => {
  const player = await CassettePlayer.load(sampleCassettePath);
  const provider = new GeminiLlmProvider({
    mode: "replay",
    player,
  });

  const controller = new AbortController();
  controller.abort();

  await assert.rejects(
    async () => {
      for await (const _ of provider.streamTurn(
        [{ role: "user", content: "Create a ground entity with a sprite and box collider." }],
        { signal: controller.signal }
      )) {
        // should abort
      }
    },
    (err: unknown) => {
      return err instanceof LlmError && err.code === "ABORTED";
    }
  );
});

// Opt-in live test tagged with { skip: ... } when GEMINI_API_KEY is unset
const hasLiveKey = Boolean(process.env.GEMINI_API_KEY);
test(
  "Live Gemini API streaming test (opt-in)",
  { skip: !hasLiveKey ? "GEMINI_API_KEY is not set" : false },
  async () => {
    const provider = new GeminiLlmProvider({
      mode: "live",
      role: "fast",
    });

    const chunks: string[] = [];
    for await (const chunk of provider.streamTurn([
      { role: "user", content: "Respond with the single word: FORGE" },
    ])) {
      if (chunk.delta) chunks.push(chunk.delta);
    }

    const fullResponse = chunks.join("").trim();
    assert.ok(fullResponse.length > 0);
  }
);
