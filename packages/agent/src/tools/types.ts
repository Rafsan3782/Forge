import type { ToolCategory, ToolRiskLevel } from "@forge/protocol";

export interface ToolMetadata {
  category: ToolCategory;
  risk: ToolRiskLevel;
  timeout?: number;
  idempotent?: boolean;
  dry_run?: boolean;
}

export const DEFAULT_MAX_OUTPUT_LENGTH = 10000;

export function truncateOutput(
  text: string,
  maxLength: number = DEFAULT_MAX_OUTPUT_LENGTH,
  hintPrefix: string = "fs.read"
): string {
  if (text.length <= maxLength) {
    return text;
  }
  const truncated = text.slice(0, maxLength);
  const remaining = text.length - maxLength;
  return `${truncated}\n\n[OUTPUT TRUNCATED: ${remaining} additional characters omitted. Use ${hintPrefix} with line or offset limits to read the remainder.]`;
}

export function isBinaryBuffer(buffer: Buffer): boolean {
  const checkLength = Math.min(buffer.length, 1024);
  for (let i = 0; i < checkLength; i++) {
    const byte = buffer[i];
    if (byte === 0) {
      return true;
    }
  }
  return false;
}
