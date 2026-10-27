import * as crypto from "node:crypto";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { LlmMessage, LlmTurnChunk } from "../contracts/interfaces.ts";
import { LlmError } from "./errors.ts";

export interface CassetteMetadata {
  recording_id: string;
  created_at: string;
  model: string;
  temperature?: number;
}

export interface CassetteRequest {
  messages: Array<{
    role: string;
    content: string;
  }>;
  available_tools?: string[];
  system_instruction?: string;
}

export interface CassetteResponse {
  thought?: string;
  tool_calls?: Array<{
    id: string;
    name: string;
    arguments: Record<string, unknown>;
  }>;
  message_delta?: string;
  finish_reason?: string;
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export interface CassetteInteraction {
  turn_id: string;
  request_hash?: string;
  request: CassetteRequest;
  response: CassetteResponse;
}

export interface Cassette {
  $schema?: string;
  version: number;
  metadata: CassetteMetadata;
  interactions: CassetteInteraction[];
}

export function hashInteractionRequest(request: CassetteRequest): string {
  const norm = {
    messages: request.messages.map((m) => ({
      role: m.role,
      content: m.content.trim(),
    })),
    tools: request.available_tools ? [...request.available_tools].sort() : [],
    sys: request.system_instruction ? request.system_instruction.trim() : "",
  };
  return crypto.createHash("sha256").update(JSON.stringify(norm)).digest("hex");
}

export class CassetteRecorder {
  private cassette: Cassette;
  private filePath: string;
  private turnCounter = 0;

  constructor(filePath: string, model: string, recordingId?: string) {
    this.filePath = filePath;
    this.cassette = {
      $schema: "https://forge.engine/schemas/cassette.json",
      version: 1,
      metadata: {
        recording_id: recordingId ?? path.basename(filePath, ".cassette.json"),
        created_at: new Date().toISOString(),
        model,
      },
      interactions: [],
    };
  }

  recordTurn(
    messages: LlmMessage[],
    tools: string[] | undefined,
    systemInstruction: string | undefined,
    chunks: LlmTurnChunk[]
  ): void {
    this.turnCounter++;
    const turnId = `turn_${String(this.turnCounter).padStart(3, "0")}`;

    const request: CassetteRequest = {
      messages: messages.map((m) => ({ role: m.role, content: m.content })),
      available_tools: tools,
      system_instruction: systemInstruction,
    };

    let accumulatedDelta = "";
    let accumulatedThought = "";
    const allToolCalls: Array<{ id: string; name: string; arguments: Record<string, unknown> }> = [];
    let finishReason: string | undefined;
    let finalUsage: CassetteResponse["usage"] = undefined;

    for (const chunk of chunks) {
      if (chunk.delta) accumulatedDelta += chunk.delta;
      if (chunk.thought) accumulatedThought += chunk.thought;
      if (chunk.tool_calls) {
        for (const tc of chunk.tool_calls) {
          allToolCalls.push({
            id: tc.id,
            name: tc.name,
            arguments: tc.arguments,
          });
        }
      }
      if (chunk.finish_reason) finishReason = chunk.finish_reason;
      if (chunk.usage) {
        finalUsage = {
          prompt_tokens: chunk.usage.input_tokens,
          completion_tokens: chunk.usage.output_tokens,
          total_tokens: chunk.usage.total_tokens,
        };
      }
    }

    const response: CassetteResponse = {
      thought: accumulatedThought || undefined,
      tool_calls: allToolCalls.length > 0 ? allToolCalls : undefined,
      message_delta: accumulatedDelta || undefined,
      finish_reason: finishReason,
      usage: finalUsage,
    };

    this.cassette.interactions.push({
      turn_id: turnId,
      request_hash: hashInteractionRequest(request),
      request,
      response,
    });
  }

  async flush(): Promise<void> {
    await fs.mkdir(path.dirname(this.filePath), { recursive: true });
    await fs.writeFile(this.filePath, JSON.stringify(this.cassette, null, 2), "utf-8");
  }
}

export class CassettePlayer {
  private cassette: Cassette;
  private interactionMap = new Map<string, CassetteInteraction>();
  private usedTurns = new Set<string>();

  constructor(cassette: Cassette) {
    this.cassette = cassette;
    for (const interaction of cassette.interactions) {
      const hash = interaction.request_hash || hashInteractionRequest(interaction.request);
      this.interactionMap.set(hash, interaction);
    }
  }

  static async load(filePath: string): Promise<CassettePlayer> {
    const raw = await fs.readFile(filePath, "utf-8");
    const parsed = JSON.parse(raw) as Cassette;
    return new CassettePlayer(parsed);
  }

  static fromObject(obj: Cassette): CassettePlayer {
    return new CassettePlayer(obj);
  }

  getModelName(): string {
    return this.cassette.metadata.model;
  }

  findInteraction(
    messages: LlmMessage[],
    tools?: string[],
    systemInstruction?: string
  ): CassetteInteraction | null {
    const req: CassetteRequest = {
      messages: messages.map((m) => ({ role: m.role, content: m.content })),
      available_tools: tools,
      system_instruction: systemInstruction,
    };
    const hash = hashInteractionRequest(req);
    const interaction = this.interactionMap.get(hash);
    if (!interaction) {
      return null;
    }
    return interaction;
  }

  async *replayTurn(
    messages: LlmMessage[],
    tools?: string[],
    systemInstruction?: string,
    signal?: AbortSignal
  ): AsyncIterable<LlmTurnChunk> {
    if (signal?.aborted) {
      throw new LlmError("Turn aborted during replay", "ABORTED");
    }

    const interaction = this.findInteraction(messages, tools, systemInstruction);
    if (!interaction) {
      throw new LlmError(
        `No cassette interaction found for prompt: "${messages[messages.length - 1]?.content ?? ""}". Available turns: ${this.cassette.interactions.map((i) => i.turn_id).join(", ")}`,
        "UNRECORDED_CASSETTE_TURN"
      );
    }

    this.usedTurns.add(interaction.turn_id);
    const resp = interaction.response;

    // Yield thought chunk if present
    if (resp.thought) {
      if (signal?.aborted) throw new LlmError("Turn aborted during replay", "ABORTED");
      yield {
        thought: resp.thought,
      };
    }

    // Yield delta chunk if present
    if (resp.message_delta) {
      if (signal?.aborted) throw new LlmError("Turn aborted during replay", "ABORTED");
      yield {
        delta: resp.message_delta,
      };
    }

    // Yield tool_calls if present
    if (resp.tool_calls && resp.tool_calls.length > 0) {
      if (signal?.aborted) throw new LlmError("Turn aborted during replay", "ABORTED");
      yield {
        tool_calls: resp.tool_calls.map((tc) => ({
          id: tc.id,
          name: tc.name,
          arguments: tc.arguments,
        })),
        finish_reason: (resp.finish_reason as "tool_calls" | "stop") || "tool_calls",
      };
    }

    // Yield usage & finish reason
    yield {
      finish_reason: (resp.finish_reason as "stop" | "tool_calls" | "length" | "error") || "stop",
      usage: resp.usage
        ? {
            input_tokens: resp.usage.prompt_tokens,
            output_tokens: resp.usage.completion_tokens,
            total_tokens: resp.usage.total_tokens,
            estimated_cost_usd: 0,
          }
        : undefined,
    };
  }
}
