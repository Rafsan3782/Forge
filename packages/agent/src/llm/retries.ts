import { LlmError, parseLlmError } from "./errors.ts";

export interface RetryOptions {
  maxAttempts?: number;
  initialDelayMs?: number;
  maxDelayMs?: number;
  backoffFactor?: number;
  signal?: AbortSignal;
  apiKey?: string;
}

export async function withRetry<T>(
  fn: () => Promise<T>,
  options: RetryOptions = {}
): Promise<T> {
  const maxAttempts = options.maxAttempts ?? 3;
  const initialDelayMs = options.initialDelayMs ?? 1000;
  const maxDelayMs = options.maxDelayMs ?? 10000;
  const backoffFactor = options.backoffFactor ?? 2;

  let attempt = 0;
  let delay = initialDelayMs;

  while (true) {
    attempt++;
    if (options.signal?.aborted) {
      throw new LlmError("Operation aborted before attempt", "ABORTED");
    }

    try {
      return await fn();
    } catch (err: unknown) {
      const llmErr = parseLlmError(err, options.apiKey);

      if (attempt >= maxAttempts || !llmErr.retryable) {
        throw llmErr;
      }

      // Check if server specified retry delay
      const waitTime = llmErr.retryAfterMs ?? Math.min(delay, maxDelayMs);
      // Add jitter (+/- 20%)
      const jitter = waitTime * (0.8 + Math.random() * 0.4);

      await sleep(jitter, options.signal);
      delay *= backoffFactor;
    }
  }
}

function sleep(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      return reject(new LlmError("Operation aborted during retry wait", "ABORTED"));
    }
    const timer = setTimeout(() => {
      resolve();
    }, ms);

    if (signal) {
      signal.addEventListener(
        "abort",
        () => {
          clearTimeout(timer);
          reject(new LlmError("Operation aborted during retry wait", "ABORTED"));
        },
        { once: true }
      );
    }
  });
}
