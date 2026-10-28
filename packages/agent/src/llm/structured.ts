import type { z } from "zod";
import type { LlmProvider, LlmMessage } from "../contracts/interfaces.ts";
import { LlmError } from "./errors.ts";

export interface StructuredOutputOptions {
  model?: string;
  temperature?: number;
  system_instruction?: string;
  signal?: AbortSignal;
}

/**
 * Generates and validates a structured JSON object against a Zod schema.
 * Supports one automatic repair retry if the output fails validation or JSON parsing.
 */
export async function generateStructuredOutput<T>(
  llm: LlmProvider,
  schema: z.ZodType<T>,
  messages: LlmMessage[],
  options: StructuredOutputOptions = {}
): Promise<T> {
  const promptMessages = [...messages];

  // Instruct model to return pure JSON
  const instruction =
    "Respond with pure JSON only matching the required structure. Do not include markdown code block formatting or commentary.";

  // First Attempt
  let rawContent = await executeLlmAccumulate(llm, promptMessages, {
    ...options,
    system_instruction: options.system_instruction
      ? `${options.system_instruction}\n\n${instruction}`
      : instruction,
  });

  let parsedJson = tryParseJson(rawContent);
  if (parsedJson) {
    const result = schema.safeParse(parsedJson);
    if (result.success) {
      return result.data;
    }
  }

  // Second Attempt (Automatic repair retry)
  const failureReason = parsedJson
    ? `Validation failed: ${schema.safeParse(parsedJson).error?.message}`
    : "Invalid JSON format: failed to parse.";

  const repairMessages: LlmMessage[] = [
    ...promptMessages,
    { role: "assistant", content: rawContent },
    {
      role: "user",
      content: `Your previous response had errors:\n${failureReason}\nPlease repair the JSON to strictly conform to the expected format and return only the valid JSON.`,
    },
  ];

  rawContent = await executeLlmAccumulate(llm, repairMessages, options);
  parsedJson = tryParseJson(rawContent);

  if (!parsedJson) {
    throw new LlmError(
      `Structured output repair failed to produce valid JSON: ${rawContent.slice(0, 200)}`,
      "UNKNOWN"
    );
  }

  const secondResult = schema.safeParse(parsedJson);
  if (!secondResult.success) {
    throw new LlmError(
      `Structured output failed schema validation after repair: ${secondResult.error.message}`,
      "UNKNOWN"
    );
  }

  return secondResult.data;
}

async function executeLlmAccumulate(
  llm: LlmProvider,
  messages: LlmMessage[],
  options: StructuredOutputOptions
): Promise<string> {
  let accumulated = "";
  for await (const chunk of llm.streamTurn(messages, {
    model: options.model,
    temperature: options.temperature,
    system_instruction: options.system_instruction,
    signal: options.signal,
  })) {
    if (chunk.delta) {
      accumulated += chunk.delta;
    }
  }
  return accumulated.trim();
}

function tryParseJson(text: string): unknown | null {
  try {
    return JSON.parse(text);
  } catch {
    // Try stripping ```json ``` markdown wrapper
    const match = text.match(/```(?:json)?\s*([\s\S]*?)\s*```/);
    if (match) {
      try {
        return JSON.parse(match[1]);
      } catch {
        return null;
      }
    }
    return null;
  }
}
