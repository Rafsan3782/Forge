import type { ToolDescriptor } from "@forge/protocol";
import type {
  DocsIndexData,
  SearchResultItem,
  ReadDocResult,
  ErrorHelpResult,
} from "./types.ts";
import { buildInMemoryDocsIndex } from "./indexer.ts";

export interface ToolResult {
  content: unknown;
  is_error: boolean;
}

export interface ToolProvider {
  listTools(): Promise<ToolDescriptor[]>;
  executeTool(
    name: string,
    args: Record<string, unknown>,
    signal?: AbortSignal
  ): Promise<ToolResult>;
}

export class DocsToolProvider implements ToolProvider {
  private indexData: DocsIndexData;

  constructor(docsRootDir?: string, customIndex?: DocsIndexData) {
    this.indexData = customIndex ?? buildInMemoryDocsIndex(docsRootDir);
  }

  public getIndexData(): DocsIndexData {
    return this.indexData;
  }

  public async listTools(): Promise<ToolDescriptor[]> {
    return [
      {
        name: "docs.search",
        description: "Search engine documentation pages and recipes by keyword query with relevance ranking",
        input_schema: {
          type: "object",
          required: ["query"],
          properties: {
            query: { type: "string", description: "Search terms or question" },
            limit: { type: "integer", description: "Max results to return (default: 5)" },
          },
        },
        category: "read",
        risk_level: "safe",
      },
      {
        name: "docs.read",
        description: "Read full or sectioned content of a documentation page by ID with token limit",
        input_schema: {
          type: "object",
          required: ["id"],
          properties: {
            id: { type: "string", description: "Document ID (e.g. 'agents.mental-model')" },
            section: { type: "string", description: "Optional section heading anchor" },
            token_limit: { type: "integer", description: "Max tokens to return (default: 2000)" },
          },
        },
        category: "read",
        risk_level: "safe",
      },
      {
        name: "docs.list_recipes",
        description: "List all cookbook recipes with IDs, summaries, tags, and token counts",
        input_schema: {
          type: "object",
          properties: {
            tag: { type: "string", description: "Optional filter by tag" },
          },
        },
        category: "read",
        risk_level: "safe",
      },
      {
        name: "docs.get_pack",
        description: "Get a self-contained agent topic pack (e.g. 'scripting-basics', 'platformer-mechanics', 'game-feel')",
        input_schema: {
          type: "object",
          required: ["topic"],
          properties: {
            topic: { type: "string", description: "Topic pack name" },
          },
        },
        category: "read",
        risk_level: "safe",
      },
      {
        name: "docs.error_help",
        description: "Explain an error code or message with root causes and actionable fixes",
        input_schema: {
          type: "object",
          required: ["code"],
          properties: {
            code: { type: "string", description: "Error code (e.g. -32001 or ERR_NO_CAMERA)" },
          },
        },
        category: "read",
        risk_level: "safe",
      },
    ];
  }

  public async executeTool(
    name: string,
    args: Record<string, unknown>,
    _signal?: AbortSignal
  ): Promise<ToolResult> {
    switch (name) {
      case "docs.search": {
        const query = String(args.query || "");
        const limit = typeof args.limit === "number" ? args.limit : 5;
        const results = this.search(query, limit);
        return { content: { results }, is_error: false };
      }

      case "docs.read": {
        const id = String(args.id || "");
        const section = args.section ? String(args.section) : undefined;
        const tokenLimit = typeof args.token_limit === "number" ? args.token_limit : 2000;
        const doc = this.read(id, section, tokenLimit);
        if (!doc) {
          return { content: `Document with id '${id}' not found`, is_error: true };
        }
        return { content: doc, is_error: false };
      }

      case "docs.list_recipes": {
        const tag = args.tag ? String(args.tag) : undefined;
        let recipes = this.indexData.recipes;
        if (tag) {
          recipes = recipes.filter((r) => r.tags.includes(tag));
        }
        return { content: { recipes, total: recipes.length }, is_error: false };
      }

      case "docs.get_pack": {
        const topic = String(args.topic || "");
        const pack = this.indexData.agent_packs[topic];
        if (!pack) {
          const available = Object.keys(this.indexData.agent_packs);
          return {
            content: `Topic pack '${topic}' not found. Available packs: ${available.join(", ")}`,
            is_error: true,
          };
        }
        return { content: { topic, content: pack }, is_error: false };
      }

      case "docs.error_help": {
        const code = String(args.code || "");
        const help = this.errorHelp(code);
        if (!help) {
          return {
            content: `No exact error entry found for '${code}'. Check rules-and-invariants for common mistakes.`,
            is_error: true,
          };
        }
        return { content: help, is_error: false };
      }

      default:
        return { content: `Unknown tool '${name}'`, is_error: true };
    }
  }

  public search(query: string, limit = 5): SearchResultItem[] {
    const rawTokens = query
      .toLowerCase()
      .replace(/[^\w\s-]/g, "")
      .split(/\s+/)
      .filter((t) => t.length > 2);

    if (rawTokens.length === 0) {
      return this.indexData.pages.slice(0, limit).map((p) => ({
        id: p.id,
        title: p.title,
        summary: p.summary,
        token_count: p.token_count,
        score: 1.0,
      }));
    }

    const scored = this.indexData.pages.map((p) => {
      let score = 0;
      const idLower = p.id.toLowerCase();
      const titleLower = p.title.toLowerCase();
      const summaryLower = p.summary.toLowerCase();
      const contentLower = p.content.toLowerCase();
      const tagsJoined = p.tags.join(" ").toLowerCase();

      for (const token of rawTokens) {
        if (idLower.includes(token)) score += 20;
        if (titleLower.includes(token)) score += 15;
        if (tagsJoined.includes(token)) score += 10;
        if (summaryLower.includes(token)) score += 8;

        const countInContent = (contentLower.match(new RegExp(`\\b${token}\\b`, "g")) || []).length;
        score += Math.min(countInContent * 2, 20);
      }

      // Exact phrase match bonus
      const qLower = query.toLowerCase();
      if (titleLower.includes(qLower)) score += 40;
      if (summaryLower.includes(qLower)) score += 25;
      if (contentLower.includes(qLower)) score += 15;

      return {
        id: p.id,
        title: p.title,
        summary: p.summary,
        token_count: p.token_count,
        score,
      };
    });

    return scored
      .filter((item) => item.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, limit);
  }

  public read(id: string, section?: string, tokenLimit = 2000): ReadDocResult | null {
    const page = this.indexData.pages.find((p) => p.id === id || p.file_path.endsWith(id) || p.file_path.endsWith(`${id}.md`));
    if (!page) return null;

    let targetText = page.content;
    if (section && page.sections) {
      const secKey = section.toLowerCase().replace(/[^\w\s-]/g, "").replace(/\s+/g, "-");
      if (page.sections[secKey]) {
        targetText = page.sections[secKey];
      }
    }

    const maxChars = tokenLimit * 4;
    const truncated = targetText.length > maxChars;
    const content = truncated ? targetText.slice(0, maxChars) + "\n\n... [TRUNCATED]" : targetText;

    return {
      id: page.id,
      title: page.title,
      section,
      content,
      token_count: Math.ceil(content.length / 4),
      truncated,
    };
  }

  public errorHelp(code: string | number): ErrorHelpResult | null {
    const key = String(code).trim();
    const entry = this.indexData.errors[key];
    if (entry) return entry;

    for (const [k, v] of Object.entries(this.indexData.errors)) {
      if (k.toLowerCase() === key.toLowerCase() || v.meaning.toLowerCase().includes(key.toLowerCase())) {
        return v;
      }
    }

    return null;
  }
}
