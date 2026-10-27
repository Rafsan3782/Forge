import fs from "node:fs";
import path from "node:path";
import type { DocsIndexData, DocPageMeta } from "./types.ts";

function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4);
}

function parseFrontmatterAndSections(content: string): {
  frontmatter: Record<string, unknown>;
  body: string;
  sections: Record<string, string>;
} {
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n([\s\S]*)$/);
  if (!match) {
    return { frontmatter: {}, body: content, sections: { default: content } };
  }

  const fmRaw = match[1];
  const body = match[2];
  const frontmatter: Record<string, unknown> = {};

  for (const line of fmRaw.split(/\r?\n/)) {
    const colonIdx = line.indexOf(":");
    if (colonIdx === -1) continue;
    const key = line.slice(0, colonIdx).trim();
    let valStr = line.slice(colonIdx + 1).trim();

    if (valStr.startsWith("[") && valStr.endsWith("]")) {
      const arrayItems = valStr
        .slice(1, -1)
        .split(",")
        .map((s) => s.trim().replace(/^["']|["']$/g, ""))
        .filter(Boolean);
      frontmatter[key] = arrayItems;
    } else {
      valStr = valStr.replace(/^["']|["']$/g, "");
      frontmatter[key] = valStr;
    }
  }

  const sections: Record<string, string> = { all: body };
  const headingMatches = [...body.matchAll(/^(#{1,3})\s+([^\r\n]+)/gm)];

  for (let i = 0; i < headingMatches.length; i++) {
    const h = headingMatches[i];
    const title = h[2].trim().toLowerCase().replace(/[^\w\s-]/g, "").replace(/\s+/g, "-");
    const startIndex = h.index ?? 0;
    const nextH = headingMatches[i + 1];
    const endIndex = nextH ? (nextH.index ?? body.length) : body.length;
    sections[title] = body.slice(startIndex, endIndex).trim();
  }

  return { frontmatter, body, sections };
}

export function buildInMemoryDocsIndex(docsRootDir?: string): DocsIndexData {
  const root = docsRootDir ?? path.resolve(process.cwd(), "docs/src");
  const pages: DocPageMeta[] = [];

  const categories: Array<"agents" | "cookbook" | "concepts"> = ["agents", "cookbook", "concepts"];

  for (const cat of categories) {
    const dir = path.join(root, cat);
    if (!fs.existsSync(dir)) continue;

    const files = fs.readdirSync(dir).filter((f) => f.endsWith(".md"));
    for (const file of files) {
      const filePath = path.join(dir, file);
      const raw = fs.readFileSync(filePath, "utf-8");
      const { frontmatter, body, sections } = parseFrontmatterAndSections(raw);

      const id = String(frontmatter.id || `${cat}.${file.replace(/\.md$/, "")}`);
      const title = String(frontmatter.title || file.replace(/\.md$/, ""));
      const summary = String(frontmatter.summary || title);
      const tags = Array.isArray(frontmatter.tags) ? (frontmatter.tags as string[]) : [];
      const audience = (frontmatter.audience as "human" | "agent" | "both") || "both";
      const engine_version = String(frontmatter.engine_version || "0.1.0");
      const last_verified = String(frontmatter.last_verified || "2026-10-26");
      const related_ids = Array.isArray(frontmatter.related_ids) ? (frontmatter.related_ids as string[]) : [];

      pages.push({
        id,
        title,
        summary,
        tags,
        audience,
        engine_version,
        last_verified,
        related_ids,
        token_count: estimateTokens(raw),
        category: cat,
        file_path: filePath,
        content: body,
        sections,
      });
    }
  }

  const recipes = pages
    .filter((p) => p.category === "cookbook")
    .map((p) => ({
      id: p.id,
      title: p.title,
      summary: p.summary,
      tags: p.tags,
      token_count: p.token_count,
      file_path: p.file_path,
    }));

  const errors: DocsIndexData["errors"] = {
    "-32001": {
      code: -32001,
      meaning: "Unauthorized",
      common_cause: "Missing or invalid bridge authentication token in handshake",
      fix: "Provide matching bearer token from .forge/bridge.json",
    },
    "-32601": {
      code: -32601,
      meaning: "Method not found",
      common_cause: "Tool name misspelled or unimplemented in bridge dispatcher",
      fix: "Query tools/list or check docs.list_tools for valid tool method names",
    },
    "-32602": {
      code: -32602,
      meaning: "Invalid params",
      common_cause: "Required parameters missing or invalid type passed to method",
      fix: "Inspect input schema for expected parameters and types",
    },
    "-32000": {
      code: -32000,
      meaning: "Engine error",
      common_cause: "Engine runtime panic, missing entity in scene, or invalid asset",
      fix: "Check entity ID existence via scene.get_tree or asset via asset.list",
    },
    "-32002": {
      code: -32002,
      meaning: "Channel closed",
      common_cause: "Engine runtime event loop terminated or shut down",
      fix: "Restart engine runtime or inspect runtime crash logs",
    },
    "ERR_NO_CAMERA": {
      code: "ERR_NO_CAMERA",
      meaning: "No active camera found in scene",
      common_cause: "Scene has no Camera2D component or camera entity deleted",
      fix: "Create an entity with Transform2D and Camera2D { is_active: true }",
    },
    "ERR_MISSING_TRANSFORM": {
      code: "ERR_MISSING_TRANSFORM",
      meaning: "Entity missing Transform2D",
      common_cause: "Sprite or collider created without a Transform2D component",
      fix: "Attach Transform2D component to host entity",
    },
    "ERR_SCRIPT_SYNTAX": {
      code: "ERR_SCRIPT_SYNTAX",
      meaning: "TypeScript check diagnostics failed",
      common_cause: "Syntax or type errors in scripts/*.ts",
      fix: "Run script.check and fix all diagnostic errors",
    },
    "ERR_AUDIO_WAV_ONLY": {
      code: "ERR_AUDIO_WAV_ONLY",
      meaning: "Audio format unsupported",
      common_cause: "Attempting to play .mp3, .ogg or uncompressed format other than 16-bit PCM WAV",
      fix: "Convert audio file to 16-bit uncompressed WAV",
    },
  };

  const agent_packs: Record<string, string> = {};

  const scriptingPages = pages.filter(
    (p) =>
      p.tags.includes("scripting") ||
      p.id.includes("script") ||
      p.id.includes("state-machines")
  );
  agent_packs["scripting-basics"] = scriptingPages
    .map((p) => `# ${p.title}\n${p.summary}\n\n${p.content}`)
    .join("\n\n---\n\n");

  const platformerPages = pages.filter(
    (p) =>
      p.tags.includes("platformer") ||
      p.tags.includes("jump") ||
      p.tags.includes("movement")
  );
  agent_packs["platformer-mechanics"] = platformerPages
    .map((p) => `# ${p.title}\n${p.summary}\n\n${p.content}`)
    .join("\n\n---\n\n");

  const physicsPages = pages.filter((p) => p.tags.includes("physics"));
  agent_packs["physics"] = physicsPages
    .map((p) => `# ${p.title}\n${p.summary}\n\n${p.content}`)
    .join("\n\n---\n\n");

  const gameFeelPages = pages.filter((p) => p.tags.includes("game-feel") || p.id.includes("feel"));
  agent_packs["game-feel"] = gameFeelPages
    .map((p) => `# ${p.title}\n${p.summary}\n\n${p.content}`)
    .join("\n\n---\n\n");

  const troubleshootingPages = pages.filter((p) => p.tags.includes("troubleshooting") || p.tags.includes("invariants") || p.tags.includes("limits"));
  agent_packs["troubleshooting"] = troubleshootingPages
    .map((p) => `# ${p.title}\n${p.summary}\n\n${p.content}`)
    .join("\n\n---\n\n");

  return {
    pages,
    recipes,
    errors,
    agent_packs,
  };
}
