export interface SecretRule {
  name: string;
  pattern: RegExp;
}

export const DEFAULT_SECRET_PATTERNS: SecretRule[] = [
  // OpenAI API Key
  { name: "OpenAI API Key", pattern: /\bsk-[A-Za-z0-9_-]{20,}\b/g },
  // Anthropic API Key
  { name: "Anthropic API Key", pattern: /\bsk-ant-[A-Za-z0-9_-]{20,}\b/g },
  // Google / Gemini API Key
  { name: "Google API Key", pattern: /\bAIza[0-9A-Za-z_-]{30,}\b/g },
  // GitHub Personal Access Token / Fine-grained token
  { name: "GitHub Token", pattern: /\b(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9_]{30,}\b/g },
  // AWS Access Key ID
  { name: "AWS Access Key", pattern: /\b(AKIA|ABIA|ACCA|ASIA)[0-9A-Z]{16}\b/g },
  // Generic Bearer Token
  { name: "Bearer Token", pattern: /\bBearer\s+[A-Za-z0-9_.-]{20,}\b/gi },
  // Generic Private Keys
  { name: "Private Key", pattern: /-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----/g },
  // Slack Token
  { name: "Slack Token", pattern: /\bxox[baprs]-[0-9]{10,13}-[0-9]{10,13}[a-zA-Z0-9-]*\b/g },
];

export class SecretFilter {
  private customPatterns: SecretRule[] = [];

  constructor(customPatterns: SecretRule[] = []) {
    this.customPatterns = customPatterns;
  }

  public redact(text: string): string {
    if (!text || typeof text !== "string") {
      return text;
    }

    let result = text;
    const allPatterns = [...DEFAULT_SECRET_PATTERNS, ...this.customPatterns];

    for (const rule of allPatterns) {
      const regex = new RegExp(rule.pattern.source, rule.pattern.flags.includes("g") ? rule.pattern.flags : rule.pattern.flags + "g");
      result = result.replace(regex, () => {
        if (rule.name === "Bearer Token") {
          return "Bearer [REDACTED]";
        }
        return `[REDACTED_${rule.name.toUpperCase().replace(/\s+/g, "_")}]`;
      });
    }

    return result;
  }

  public redactObject<T>(obj: T): T {
    if (obj === null || obj === undefined) {
      return obj;
    }
    if (typeof obj === "string") {
      return this.redact(obj) as unknown as T;
    }
    if (Array.isArray(obj)) {
      return obj.map((item) => this.redactObject(item)) as unknown as T;
    }
    if (typeof obj === "object") {
      const cloned: Record<string, unknown> = {};
      for (const [k, v] of Object.entries(obj)) {
        const redactedKey = this.redact(k);
        cloned[redactedKey] = this.redactObject(v);
      }
      return cloned as unknown as T;
    }
    return obj;
  }
}
