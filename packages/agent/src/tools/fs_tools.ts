import { ProjectSandbox } from "./fs_sandbox.ts";
import type { ToolDescriptor } from "@forge/protocol";
import type { ToolMetadata } from "./types.ts";

export interface ToolDefinition {
  descriptor: ToolDescriptor;
  metadata: ToolMetadata;
  execute: (args: Record<string, unknown>, signal?: AbortSignal) => Promise<unknown>;
}

export function createFsTools(sandbox: ProjectSandbox): ToolDefinition[] {
  return [
    {
      descriptor: {
        name: "fs.read",
        description: "Read text contents of a file within the project sandbox with optional line ranges.",
        category: "read",
        risk_level: "safe",
        input_schema: {
          type: "object",
          required: ["path"],
          properties: {
            path: { type: "string", description: "Relative path to the file within the project" },
            start_line: { type: "integer", description: "Optional starting line (1-indexed)" },
            end_line: { type: "integer", description: "Optional ending line (1-indexed, inclusive)" },
          },
        },
      },
      metadata: {
        category: "read",
        risk: "safe",
        idempotent: true,
      },
      execute: async (args) => {
        const filePath = String(args.path ?? "");
        const startLine = args.start_line !== undefined ? Number(args.start_line) : undefined;
        const endLine = args.end_line !== undefined ? Number(args.end_line) : undefined;
        return sandbox.readFile(filePath, { start_line: startLine, end_line: endLine });
      },
    },
    {
      descriptor: {
        name: "fs.write",
        description: "Create or overwrite a file with UTF-8 text within the project sandbox.",
        category: "write",
        risk_level: "medium",
        input_schema: {
          type: "object",
          required: ["path", "content"],
          properties: {
            path: { type: "string", description: "Relative path to the file" },
            content: { type: "string", description: "UTF-8 content to write" },
          },
        },
      },
      metadata: {
        category: "write",
        risk: "medium",
        idempotent: true,
      },
      execute: async (args) => {
        const filePath = String(args.path ?? "");
        const content = typeof args.content === "string" ? args.content : String(args.content ?? "");
        return sandbox.writeFile(filePath, content);
      },
    },
    {
      descriptor: {
        name: "fs.edit",
        description: "Perform an exact search-and-replace on a file. Fails if search text is not found or matches more than once.",
        category: "write",
        risk_level: "medium",
        input_schema: {
          type: "object",
          required: ["path", "search_text", "replace_text"],
          properties: {
            path: { type: "string", description: "Relative path to the file" },
            search_text: { type: "string", description: "Exact text chunk to find in the file (must match exactly once)" },
            replace_text: { type: "string", description: "Replacement text" },
          },
        },
      },
      metadata: {
        category: "write",
        risk: "medium",
        idempotent: false,
      },
      execute: async (args) => {
        const filePath = String(args.path ?? "");
        const searchText = String(args.search_text ?? "");
        const replaceText = String(args.replace_text ?? "");
        return sandbox.editFile(filePath, searchText, replaceText);
      },
    },
    {
      descriptor: {
        name: "fs.list",
        description: "List contents of a directory in the project.",
        category: "read",
        risk_level: "safe",
        input_schema: {
          type: "object",
          properties: {
            path: { type: "string", description: "Relative directory path (default: .)" },
          },
        },
      },
      metadata: {
        category: "read",
        risk: "safe",
        idempotent: true,
      },
      execute: async (args) => {
        const dirPath = args.path !== undefined ? String(args.path) : ".";
        return sandbox.listDirectory(dirPath);
      },
    },
    {
      descriptor: {
        name: "fs.glob",
        description: "Find files matching a glob pattern (e.g. **/*.ts, scripts/*) within the project.",
        category: "read",
        risk_level: "safe",
        input_schema: {
          type: "object",
          required: ["pattern"],
          properties: {
            pattern: { type: "string", description: "Glob pattern to match" },
            base_dir: { type: "string", description: "Base directory to start search from (default: .)" },
          },
        },
      },
      metadata: {
        category: "read",
        risk: "safe",
        idempotent: true,
      },
      execute: async (args) => {
        const pattern = String(args.pattern ?? "");
        const baseDir = args.base_dir !== undefined ? String(args.base_dir) : ".";
        return sandbox.glob(pattern, baseDir);
      },
    },
    {
      descriptor: {
        name: "fs.search",
        description: "Search for a regex or string pattern across files in the project with context lines.",
        category: "read",
        risk_level: "safe",
        input_schema: {
          type: "object",
          required: ["query"],
          properties: {
            query: { type: "string", description: "Search term or regex pattern" },
            path: { type: "string", description: "Optional file or directory path to restrict search" },
            context_lines: { type: "integer", description: "Number of context lines before and after match (default: 2)" },
            is_regex: { type: "boolean", description: "Whether query should be treated as a regular expression (default: false)" },
          },
        },
      },
      metadata: {
        category: "read",
        risk: "safe",
        idempotent: true,
      },
      execute: async (args) => {
        const query = String(args.query ?? "");
        const searchPath = args.path !== undefined ? String(args.path) : undefined;
        const contextLines = args.context_lines !== undefined ? Number(args.context_lines) : 2;
        const isRegex = args.is_regex === true;
        return sandbox.search(query, { path: searchPath, context_lines: contextLines, is_regex: isRegex });
      },
    },
    {
      descriptor: {
        name: "fs.move",
        description: "Move or rename a file or directory within the project.",
        category: "write",
        risk_level: "medium",
        input_schema: {
          type: "object",
          required: ["source", "target"],
          properties: {
            source: { type: "string", description: "Source path" },
            target: { type: "string", description: "Target destination path" },
          },
        },
      },
      metadata: {
        category: "write",
        risk: "medium",
        idempotent: false,
      },
      execute: async (args) => {
        const source = String(args.source ?? "");
        const target = String(args.target ?? "");
        return sandbox.move(source, target);
      },
    },
    {
      descriptor: {
        name: "fs.delete",
        description: "Delete a file or directory within the project.",
        category: "write",
        risk_level: "high",
        input_schema: {
          type: "object",
          required: ["path"],
          properties: {
            path: { type: "string", description: "Path of file or directory to remove" },
          },
        },
      },
      metadata: {
        category: "write",
        risk: "high",
        idempotent: false,
      },
      execute: async (args) => {
        const targetPath = String(args.path ?? "");
        return sandbox.delete(targetPath);
      },
    },
  ];
}
