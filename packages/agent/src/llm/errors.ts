export type LlmErrorCode =
  | "INVALID_KEY"
  | "QUOTA_EXCEEDED"
  | "SAFETY_BLOCKED"
  | "CONTEXT_OVERFLOW"
  | "RATE_LIMITED"
  | "SERVER_ERROR"
  | "ABORTED"
  | "UNRECORDED_CASSETTE_TURN"
  | "UNKNOWN";

export class LlmError extends Error {
  readonly code: LlmErrorCode;
  readonly status?: number;
  readonly retryable: boolean;
  readonly retryAfterMs?: number;

  constructor(
    message: string,
    code: LlmErrorCode,
    options?: {
      status?: number;
      retryable?: boolean;
      retryAfterMs?: number;
      cause?: unknown;
    }
  ) {
    super(message);
    this.name = "LlmError";
    this.code = code;
    this.status = options?.status;
    this.retryable = options?.retryable ?? false;
    this.retryAfterMs = options?.retryAfterMs;
    if (options?.cause) {
      this.cause = options.cause;
    }
  }
}

export function parseLlmError(err: unknown, keyToRedact?: string): LlmError {
  if (err instanceof LlmError) {
    return err;
  }

  const rawMsg =
    err instanceof Error
      ? err.message
      : typeof (err as { message?: unknown })?.message === "string"
      ? (err as { message: string }).message
      : String(err);
  const msg = keyToRedact ? redactKey(rawMsg, keyToRedact) : rawMsg;

  // Extract status if available
  const status = (err as { status?: number })?.status;

  // Key detection
  if (
    status === 401 ||
    status === 403 ||
    /API key not valid|invalid api key|PERMISSION_DENIED|unauthorized/i.test(msg)
  ) {
    return new LlmError(
      "Invalid or unauthorized Gemini API key.",
      "INVALID_KEY",
      { status: status ?? 401, retryable: false, cause: err }
    );
  }

  // Quota & Rate Limit
  if (status === 429 || /RESOURCE_EXHAUSTED|quota exceeded|too many requests/i.test(msg)) {
    // Check if it's general rate limit vs hard quota
    const isHardQuota = /quota exceeded/i.test(msg) && !/per minute/i.test(msg);
    return new LlmError(
      isHardQuota ? "Gemini quota exceeded." : "Gemini rate limit exceeded.",
      isHardQuota ? "QUOTA_EXCEEDED" : "RATE_LIMITED",
      { status: 429, retryable: !isHardQuota, cause: err }
    );
  }

  // Safety Block
  if (/SAFETY|blocked reason: SAFETY|HARM_CATEGORY/i.test(msg)) {
    return new LlmError(
      "Gemini generation was blocked by safety filters.",
      "SAFETY_BLOCKED",
      { status, retryable: false, cause: err }
    );
  }

  // Context Overflow
  if (/context length|maximum context|token limit exceeded|input token count exceeds/i.test(msg)) {
    return new LlmError(
      "Input prompt exceeds maximum context length for this model.",
      "CONTEXT_OVERFLOW",
      { status, retryable: false, cause: err }
    );
  }

  // 5xx Server Error
  if (status && status >= 500 && status < 600) {
    return new LlmError(
      `Gemini server error (${status}).`,
      "SERVER_ERROR",
      { status, retryable: true, cause: err }
    );
  }

  // Abort
  if (/abort|canceled|cancelled/i.test(msg)) {
    return new LlmError("LLM operation was aborted.", "ABORTED", {
      retryable: false,
      cause: err,
    });
  }

  return new LlmError(msg, "UNKNOWN", { status, retryable: false, cause: err });
}

export function redactKey(text: string, apiKey?: string): string {
  if (!apiKey || apiKey.length < 4) {
    return text.replace(/AIza[0-9A-Za-z-_]{35}/g, "[REDACTED_API_KEY]");
  }
  let result = text.replaceAll(apiKey, "[REDACTED_API_KEY]");
  // Also regex match standard Google API key patterns (AIzaSy...)
  result = result.replace(/AIza[0-9A-Za-z-_]{35}/g, "[REDACTED_API_KEY]");
  return result;
}
