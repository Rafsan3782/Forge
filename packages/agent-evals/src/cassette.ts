import * as fs from "node:fs/promises";
import * as path from "node:path";
import type {
  LlmProvider,
  LlmMessage,
  LlmTurnChunk,
  LlmTurnOptions,
} from "@forge/agent";
import type { CassetteData, CassetteTurnInteraction } from "./types.ts";

/**
 * Normalizes input messages for matching against recorded cassette interactions.
 */
function hashRequest(
  messages: LlmMessage[],
  tools?: { name: string }[]
): string {
  const simplifiedMsgs = messages.map((m) => ({
    role: m.role,
    content: m.content.trim(),
    name: m.name,
    tool_call_id: m.tool_call_id,
  }));
  const toolNames = (tools || []).map((t) => t.name).sort();
  return JSON.stringify({ messages: simplifiedMsgs, tools: toolNames });
}

/**
 * ReplayLlmProvider deterministically serves recorded turns from a .cassette.json file.
 */
export class ReplayLlmProvider implements LlmProvider {
  private cassette: CassetteData;
  private turnIndex = 0;
  private recordedCalls: Array<{ messages: LlmMessage[]; options?: LlmTurnOptions }> = [];

  constructor(cassette: CassetteData) {
    this.cassette = cassette;
  }

  public static async fromFile(filePath: string): Promise<ReplayLlmProvider> {
    const raw = await fs.readFile(filePath, "utf-8");
    const cassette = JSON.parse(raw) as CassetteData;
    return new ReplayLlmProvider(cassette);
  }

  public getModelName(): string {
    return this.cassette.metadata.model || "gemini-2.5-pro";
  }

  public getRecordedCalls() {
    return this.recordedCalls;
  }

  public async *streamTurn(
    messages: LlmMessage[],
    options?: LlmTurnOptions
  ): AsyncIterable<LlmTurnChunk> {
    this.recordedCalls.push({ messages, options });

    if (this.turnIndex >= this.cassette.interactions.length) {
      throw new Error(
        `Replay cassette '${this.cassette.metadata.recording_id}' exhausted. Requested turn ${
          this.turnIndex + 1
        }, but cassette only contains ${this.cassette.interactions.length} turn(s).`
      );
    }

    const interaction = this.cassette.interactions[this.turnIndex++];
    const resp = interaction.response;

    if (resp.thought) {
      yield { thought: resp.thought };
    }

    if (resp.tool_calls && resp.tool_calls.length > 0) {
      yield {
        tool_calls: resp.tool_calls,
        finish_reason: "tool_calls",
        usage: resp.usage,
      };
      return;
    }

    if (resp.message_delta) {
      yield {
        delta: resp.message_delta,
        finish_reason: resp.finish_reason || "stop",
        usage: resp.usage,
      };
      return;
    }

    yield {
      delta: "Step completed.",
      finish_reason: "stop",
      usage: resp.usage || {
        input_tokens: 50,
        output_tokens: 20,
        total_tokens: 70,
        estimated_cost_usd: 0.0001,
      },
    };
  }
}

/**
 * LiveRecordingLlmProvider wraps a live LlmProvider (e.g. Gemini) and records all turns into a CassetteData structure.
 */
export class LiveRecordingLlmProvider implements LlmProvider {
  private baseProvider: LlmProvider;
  private recordingId: string;
  private cassette: CassetteData;
  private turnCount = 0;

  constructor(baseProvider: LlmProvider, recordingId: string) {
    this.baseProvider = baseProvider;
    this.recordingId = recordingId;
    this.cassette = {
      $schema: "https://forge.engine/schemas/cassette.json",
      version: 1,
      metadata: {
        recording_id: recordingId,
        created_at: new Date().toISOString(),
        model: baseProvider.getModelName(),
        temperature: 0.2,
      },
      interactions: [],
    };
  }

  public getModelName(): string {
    return this.baseProvider.getModelName();
  }

  public getCassette(): CassetteData {
    return this.cassette;
  }

  public async saveToFile(filePath: string): Promise<void> {
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, JSON.stringify(this.cassette, null, 2), "utf-8");
  }

  public async *streamTurn(
    messages: LlmMessage[],
    options?: LlmTurnOptions
  ): AsyncIterable<LlmTurnChunk> {
    const turnId = `turn_${String(++this.turnCount).padStart(3, "0")}`;
    const interaction: CassetteTurnInteraction = {
      turn_id: turnId,
      request: {
        messages: messages.map((m) => ({
          role: m.role,
          content: m.content,
          name: m.name,
          tool_call_id: m.tool_call_id,
        })),
        available_tools: (options?.tools || []).map((t) => t.name),
      },
      response: {
        tool_calls: [],
      },
    };

    let accumulatedDelta = "";
    let accumulatedThought = "";

    for await (const chunk of this.baseProvider.streamTurn(messages, options)) {
      if (chunk.thought) {
        accumulatedThought += chunk.thought;
      }
      if (chunk.delta) {
        accumulatedDelta += chunk.delta;
      }
      if (chunk.tool_calls) {
        interaction.response.tool_calls = [
          ...(interaction.response.tool_calls || []),
          ...chunk.tool_calls,
        ];
      }
      if (chunk.finish_reason) {
        interaction.response.finish_reason = chunk.finish_reason;
      }
      if (chunk.usage) {
        interaction.response.usage = chunk.usage;
      }
      yield chunk;
    }

    if (accumulatedThought) {
      interaction.response.thought = accumulatedThought;
    }
    if (accumulatedDelta) {
      interaction.response.message_delta = accumulatedDelta;
    }

    this.cassette.interactions.push(interaction);
  }
}
