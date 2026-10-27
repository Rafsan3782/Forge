import { z } from "zod";

export const DocsAudienceSchema = z.enum(["human", "agent", "both"]);
export type DocsAudience = z.infer<typeof DocsAudienceSchema>;

export const DocsFrontmatterSchema = z.object({
  id: z.string().min(1, "Document id must not be empty"),
  title: z.string().min(1, "Title must not be empty"),
  summary: z.string().min(5, "Summary must be at least 5 characters and end with a period"),
  tags: z.array(z.string()).min(1, "At least one tag is required"),
  audience: DocsAudienceSchema,
  engine_version: z.string().regex(/^\d+\.\d+\.\d+/, "engine_version must follow semver (e.g. '0.1.0')"),
  last_verified: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, "last_verified must be YYYY-MM-DD"),
  related_ids: z.array(z.string()).default([]),
});

export type DocsFrontmatter = z.infer<typeof DocsFrontmatterSchema>;

export interface ParsedDocPage {
  filePath: string;
  frontmatter: DocsFrontmatter;
  rawContent: string;
  bodyContent: string;
  tokenCount: number;
}

export function parseFrontmatter(rawContent: string, filePath: string): { frontmatter: DocsFrontmatter; bodyContent: string } {
  const match = rawContent.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n([\s\S]*)$/);
  if (!match) {
    throw new Error(`File '${filePath}' does not contain valid YAML frontmatter delimiters ('---').`);
  }

  const rawYaml = match[1];
  const bodyContent = match[2];
  const fields: Record<string, unknown> = {};

  for (const line of rawYaml.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;

    const colonIdx = trimmed.indexOf(":");
    if (colonIdx === -1) continue;

    const key = trimmed.slice(0, colonIdx).trim();
    let valStr = trimmed.slice(colonIdx + 1).trim();

    // Parse JSON-like arrays or string primitives
    if (valStr.startsWith("[") && valStr.endsWith("]")) {
      try {
        fields[key] = JSON.parse(valStr);
      } catch {
        const inner = valStr.slice(1, -1).trim();
        fields[key] = inner ? inner.split(",").map(s => s.trim().replace(/^["']|["']$/g, "")) : [];
      }
    } else {
      if ((valStr.startsWith('"') && valStr.endsWith('"')) || (valStr.startsWith("'") && valStr.endsWith("'"))) {
        valStr = valStr.slice(1, -1);
      }
      fields[key] = valStr;
    }
  }

  const result = DocsFrontmatterSchema.safeParse(fields);
  if (!result.success) {
    throw new Error(`Frontmatter validation failed for '${filePath}':\n${result.error.issues.map(i => ` - ${i.path.join(".")}: ${i.message}`).join("\n")}`);
  }

  return { frontmatter: result.data, bodyContent };
}

export function estimateTokens(text: string): number {
  const words = text.trim().split(/\s+/).filter(Boolean);
  const byWord = Math.ceil(words.length * 1.3);
  const byChar = Math.ceil(text.length / 4);
  return Math.max(byWord, byChar);
}
