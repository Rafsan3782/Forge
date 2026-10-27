export type DocsAudience = "human" | "agent" | "both";

export interface DocsFrontmatter {
  id: string;
  title: string;
  summary: string;
  tags: string[];
  audience: DocsAudience;
  engine_version: string;
  last_verified: string;
  related_ids: string[];
}

export interface ParsedDocPage {
  filePath: string;
  frontmatter: DocsFrontmatter;
  rawContent: string;
  bodyContent: string;
  tokenCount: number;
}

export function validateFrontmatter(fields: Record<string, unknown>, filePath: string): DocsFrontmatter {
  const errors: string[] = [];

  if (typeof fields.id !== "string" || !fields.id.trim()) {
    errors.push("Document 'id' must be a non-empty string");
  }

  if (typeof fields.title !== "string" || !fields.title.trim()) {
    errors.push("Document 'title' must be a non-empty string");
  }

  if (typeof fields.summary !== "string" || fields.summary.trim().length < 5) {
    errors.push("Document 'summary' must be at least 5 characters");
  } else if (!fields.summary.trim().endsWith(".")) {
    errors.push("Document 'summary' should end with a period");
  }

  if (!Array.isArray(fields.tags) || fields.tags.length === 0) {
    errors.push("Document 'tags' must be a non-empty array of strings");
  }

  const validAudiences = ["human", "agent", "both"];
  if (typeof fields.audience !== "string" || !validAudiences.includes(fields.audience)) {
    errors.push("Document 'audience' must be one of: 'human', 'agent', 'both'");
  }

  if (typeof fields.engine_version !== "string" || !/^\d+\.\d+\.\d+/.test(fields.engine_version)) {
    errors.push("Document 'engine_version' must follow semver (e.g. '0.1.0')");
  }

  if (typeof fields.last_verified !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(fields.last_verified)) {
    errors.push("Document 'last_verified' must be YYYY-MM-DD");
  }

  const related_ids: string[] = Array.isArray(fields.related_ids)
    ? fields.related_ids.map(s => String(s).trim())
    : [];

  if (errors.length > 0) {
    throw new Error(`Frontmatter validation failed for '${filePath}':\n${errors.map(e => ` - ${e}`).join("\n")}`);
  }

  return {
    id: String(fields.id).trim(),
    title: String(fields.title).trim(),
    summary: String(fields.summary).trim(),
    tags: (fields.tags as unknown[]).map(t => String(t).trim()),
    audience: fields.audience as DocsAudience,
    engine_version: String(fields.engine_version).trim(),
    last_verified: String(fields.last_verified).trim(),
    related_ids,
  };
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

  const validated = validateFrontmatter(fields, filePath);
  return { frontmatter: validated, bodyContent };
}

export function estimateTokens(text: string): number {
  const words = text.trim().split(/\s+/).filter(Boolean);
  const byWord = Math.ceil(words.length * 1.3);
  const byChar = Math.ceil(text.length / 4);
  return Math.max(byWord, byChar);
}
