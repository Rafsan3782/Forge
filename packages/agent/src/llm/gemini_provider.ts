import { GoogleGenAI } from "@google/genai";
import type { ToolDescriptor } from "@forge/protocol";
import type {
  LlmProvider,
  LlmMessage,
  LlmTurnChunk,
  LlmTurnOptions,
  LlmToolCall,
} from "../contracts/interfaces.ts";
import { getRoleConfig, estimateCost, type ModelRole } from "./models_config.ts";
import { parseLlmError, redactKey, LlmError } from "./errors.ts";
import { withRetry } from "./retries.ts";
import { CassettePlayer, CassetteRecorder } from "./cassette.ts";
import { ContextCacheManager } from "./context_cache.ts";

export type ForgeLlmMode = "live" | "record" | "replay" | "mock";

export interface GeminiProviderOptions {
  apiKey?: string;
  role?: ModelRole;
  mode?: ForgeLlmMode;
  cassettePath?: string;
  player?: CassettePlayer;
  recorder?: CassetteRecorder;
  projectBrainPrefix?: string;
}

export class GeminiLlmProvider implements LlmProvider {
  private apiKey?: string;
  private role: ModelRole;
  private mode: ForgeLlmMode;
  private client?: GoogleGenAI;
  private player?: CassettePlayer;
  private recorder?: CassetteRecorder;
  private cacheManager?: ContextCacheManager;
  private projectBrainPrefix?: string;

  constructor(options: GeminiProviderOptions = {}) {
    this.role = options.role ?? "planner";
    this.apiKey = options.apiKey ?? process.env.GEMINI_API_KEY;

    const envMode = process.env.FORGE_LLM_MODE?.toLowerCase() as ForgeLlmMode | undefined;
    this.mode = options.mode ?? envMode ?? (this.apiKey ? "live" : "replay");

    this.projectBrainPrefix = options.projectBrainPrefix;

    if (this.mode === "live" || this.mode === "record") {
      if (!this.apiKey) {
        throw new LlmError(
          "Gemini API key is required for live/record mode.",
          "INVALID_KEY"
        );
      }
      if (this.apiKey) {
        this.client = new GoogleGenAI({ apiKey: this.apiKey });
        this.cacheManager = new ContextCacheManager(this.client);
      }
    }

    if (options.player) {
      this.player = options.player;
    }

    if (options.recorder) {
      this.recorder = options.recorder;
    } else if (this.mode === "record" && options.cassettePath) {
      const roleCfg = getRoleConfig(this.role);
      this.recorder = new CassetteRecorder(options.cassettePath, roleCfg.model);
    }
  }

  getModelName(): string {
    if (this.player) {
      return this.player.getModelName();
    }
    const roleCfg = getRoleConfig(this.role);
    return roleCfg.model;
  }

  getMode(): ForgeLlmMode {
    return this.mode;
  }

  async *streamTurn(
    messages: LlmMessage[],
    options: LlmTurnOptions = {}
  ): AsyncIterable<LlmTurnChunk> {
    if (options.signal?.aborted) {
      throw new LlmError("Turn aborted before execution", "ABORTED");
    }

    // 1. Replay mode
    if (this.mode === "replay") {
      if (!this.player) {
        throw new LlmError(
          "Replay mode requested but no cassette player provided.",
          "UNRECORDED_CASSETTE_TURN"
        );
      }
      const toolNames = options.tools?.map((t) => t.name);
      for await (const chunk of this.player.replayTurn(
        messages,
        toolNames,
        options.system_instruction,
        options.signal
      )) {
        yield chunk;
      }
      return;
    }

    // 2. Mock mode (delegates to deterministic generator)
    if (this.mode === "mock") {
      yield { thought: "Mock thinking..." };
      yield { delta: "Mock answer." };
      yield {
        finish_reason: "stop",
        usage: {
          input_tokens: 10,
          output_tokens: 5,
          total_tokens: 15,
          estimated_cost_usd: 0,
        },
      };
      return;
    }

    // 3. Live or Record mode
    if (!this.client) {
      throw new LlmError("GoogleGenAI client not initialized.", "INVALID_KEY");
    }

    const roleCfg = getRoleConfig(this.role);
    const model = options.model ?? roleCfg.model;
    const recordedChunks: LlmTurnChunk[] = [];

    // Format tools if provided
    let functionDeclarations: Array<Record<string, unknown>> | undefined;
    if (options.tools && options.tools.length > 0) {
      functionDeclarations = options.tools.map((t) => ({
        name: t.name,
        description: t.description,
        parametersJsonSchema: t.input_schema,
      }));
    }

    // Check context caching
    let cachedContentResource: string | undefined;
    if (this.cacheManager) {
      cachedContentResource = await this.cacheManager.getOrOrCreateCache({
        model,
        systemInstruction: options.system_instruction,
        toolDeclarations: functionDeclarations,
        brainContent: this.projectBrainPrefix,
      });
    }

    // Format Gemini contents
    const contents = this.formatMessagesForGemini(messages);

    const config: Record<string, unknown> = {
      temperature: options.temperature ?? roleCfg.generation.temperature,
      maxOutputTokens: options.max_output_tokens ?? roleCfg.generation.max_output_tokens,
      abortSignal: options.signal,
    };

    if (cachedContentResource) {
      config.cachedContent = cachedContentResource;
    } else if (options.system_instruction) {
      config.systemInstruction = options.system_instruction;
    }

    if (functionDeclarations && functionDeclarations.length > 0) {
      config.tools = [{ functionDeclarations }];
    }

    if (roleCfg.thinking.budget > 0) {
      config.thinkingConfig = {
        thinkingBudget: roleCfg.thinking.budget,
        includeThoughts: roleCfg.thinking.include_thoughts,
      };
    } else if (roleCfg.thinking.budget === 0) {
      config.thinkingConfig = {
        thinkingBudget: 0,
      };
    }

    // Call API with retries
    const stream = await withRetry(
      async () => {
        try {
          return await this.client!.models.generateContentStream({
            model,
            contents,
            config,
          });
        } catch (err: unknown) {
          throw parseLlmError(err, this.apiKey);
        }
      },
      {
        signal: options.signal,
        apiKey: this.apiKey,
      }
    );

    let promptTokens = 0;
    let completionTokens = 0;
    let cachedTokens = 0;

    for await (const chunk of stream) {
      if (options.signal?.aborted) {
        throw new LlmError("Turn aborted during stream", "ABORTED");
      }

      // Check for usage metadata
      if (chunk.usageMetadata) {
        promptTokens = chunk.usageMetadata.promptTokenCount ?? promptTokens;
        completionTokens = chunk.usageMetadata.candidatesTokenCount ?? completionTokens;
        const usage = chunk.usageMetadata as Record<string, unknown>;
        if (typeof usage.cachedContentTokenCount === "number") {
          cachedTokens = usage.cachedContentTokenCount;
        }
      }

      const candidate = chunk.candidates?.[0];
      const finishReasonRaw = candidate?.finishReason;

      let mappedFinishReason: "stop" | "tool_calls" | "length" | "error" | undefined;
      if (finishReasonRaw === "STOP") mappedFinishReason = "stop";
      else if (finishReasonRaw === "MAX_TOKENS") mappedFinishReason = "length";
      else if (finishReasonRaw === "SAFETY" || finishReasonRaw === "BLOCKLIST") {
        throw new LlmError("Generation blocked by safety filters", "SAFETY_BLOCKED");
      }

      const parts = candidate?.content?.parts;
      if (parts && parts.length > 0) {
        const toolCallsInChunk: LlmToolCall[] = [];

        for (const part of parts) {
          const partObj = part as Record<string, unknown>;
          if (partObj.thought) {
            const thoughtChunk: LlmTurnChunk = { thought: part.text };
            recordedChunks.push(thoughtChunk);
            yield thoughtChunk;
          } else if (part.text) {
            const textChunk: LlmTurnChunk = { delta: part.text };
            recordedChunks.push(textChunk);
            yield textChunk;
          }

          if (part.functionCall && part.functionCall.name) {
            toolCallsInChunk.push({
              id: `call_${Math.random().toString(36).slice(2, 9)}`,
              name: part.functionCall.name,
              arguments: (part.functionCall.args as Record<string, unknown>) ?? {},
            });
          }
        }

        if (toolCallsInChunk.length > 0) {
          const tcChunk: LlmTurnChunk = {
            tool_calls: toolCallsInChunk,
            finish_reason: "tool_calls",
          };
          recordedChunks.push(tcChunk);
          yield tcChunk;
        }
      }

      if (mappedFinishReason && mappedFinishReason !== "tool_calls") {
        const finishChunk: LlmTurnChunk = {
          finish_reason: mappedFinishReason,
        };
        recordedChunks.push(finishChunk);
        yield finishChunk;
      }
    }

    // Final usage chunk
    const totalTokens = promptTokens + completionTokens;
    const estCost = estimateCost(model, promptTokens, completionTokens, cachedTokens);

    const usageChunk: LlmTurnChunk = {
      usage: {
        input_tokens: promptTokens,
        output_tokens: completionTokens,
        total_tokens: totalTokens,
        estimated_cost_usd: estCost,
      },
    };
    recordedChunks.push(usageChunk);
    yield usageChunk;

    // Record turn if recorder active
    if (this.recorder) {
      this.recorder.recordTurn(
        messages,
        options.tools?.map((t) => t.name),
        options.system_instruction,
        recordedChunks
      );
      await this.recorder.flush();
    }
  }

  private formatMessagesForGemini(
    messages: LlmMessage[]
  ): Array<{ role: string; parts: Array<Record<string, unknown>> }> {
    const formatted: Array<{ role: string; parts: Array<Record<string, unknown>> }> = [];

    for (const msg of messages) {
      if (msg.role === "system") {
        // System instruction handled in config
        continue;
      }

      if (msg.role === "user") {
        formatted.push({
          role: "user",
          parts: [{ text: msg.content }],
        });
      } else if (msg.role === "assistant") {
        formatted.push({
          role: "model",
          parts: [{ text: msg.content }],
        });
      } else if (msg.role === "tool") {
        formatted.push({
          role: "user",
          parts: [
            {
              functionResponse: {
                name: msg.name ?? "tool_result",
                response: { output: msg.content },
              },
            },
          ],
        });
      }
    }

    return formatted;
  }
}
