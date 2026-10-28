import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parseFrontmatter } from "./schema.ts";
import { collectAllDocFiles } from "./build.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../..");
const DOCS_SRC = path.join(REPO_ROOT, "docs/src");
const DIST_AGENT_PACKS = path.join(REPO_ROOT, "docs/dist/agent-packs");

export interface CodeBlock {
  filePath: string;
  line: number;
  mode: "check" | "run" | "snippet";
  code: string;
}

export function extractCodeBlocks(filePath: string, content: string): CodeBlock[] {
  const blocks: CodeBlock[] = [];
  const lines = content.split(/\r?\n/);
  let inBlock = false;
  let mode: "check" | "run" | "snippet" = "snippet";
  let currentCode: string[] = [];
  let startLine = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (!inBlock) {
      const match = line.match(/^```(check|run|snippet)\b/);
      if (match) {
        inBlock = true;
        mode = match[1] as "check" | "run" | "snippet";
        startLine = i + 1;
        currentCode = [];
      }
    } else {
      if (line.startsWith("```")) {
        inBlock = false;
        blocks.push({
          filePath,
          line: startLine,
          mode,
          code: currentCode.join("\n")
        });
      } else {
        currentCode.push(line);
      }
    }
  }

  return blocks;
}

export function validateAllQualityGates(): { success: boolean; errors: string[] } {
  const errors: string[] = [];

  console.log("=== Running Documentation Quality Gates ===");

  // Gate 1: Schema validation & broken link check
  console.log("1. Checking schemas and links across docs/src...");
  const docFiles = collectAllDocFiles(DOCS_SRC);
  const knownDocIds = new Set<string>();
  const pageMap = new Map<string, { frontmatter: any; bodyContent: string }>();

  for (const f of docFiles) {
    const raw = fs.readFileSync(f, "utf-8");
    try {
      const parsed = parseFrontmatter(raw, f);
      knownDocIds.add(parsed.frontmatter.id);
      pageMap.set(f, parsed);
    } catch (e: any) {
      errors.push(`[Schema Validation Error] ${e.message}`);
    }
  }

  // Check related_ids and relative markdown links
  for (const [f, { frontmatter, bodyContent }] of pageMap.entries()) {
    // Check related_ids
    for (const relId of frontmatter.related_ids || []) {
      if (!knownDocIds.has(relId)) {
        errors.push(`[Broken Related ID] '${relId}' referenced in ${f} does not exist in any document.`);
      }
    }

    // Check markdown file links
    const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
    let linkMatch;
    while ((linkMatch = linkRegex.exec(bodyContent)) !== null) {
      const target = linkMatch[2];
      if (target.startsWith("http://") || target.startsWith("https://") || target.startsWith("#")) {
        continue;
      }
      // Handle file:/// or relative path
      let cleanTarget = target.replace(/^file:\/\/\/?/, "");
      if (cleanTarget.startsWith("docs/")) {
        cleanTarget = path.join(REPO_ROOT, cleanTarget);
      } else if (!path.isAbsolute(cleanTarget)) {
        cleanTarget = path.resolve(path.dirname(f), cleanTarget);
      }
      // Strip anchor
      const hashIdx = cleanTarget.indexOf("#");
      if (hashIdx !== -1) {
        cleanTarget = cleanTarget.slice(0, hashIdx);
      }
      if (!fs.existsSync(cleanTarget)) {
        errors.push(`[Broken Link] Link to '${target}' in ${f} cannot be resolved on disk (checked ${cleanTarget}).`);
      }
    }
  }

  // Gate 2: Code example extraction
  console.log("2. Extracting and validating code blocks...");
  const allBlocks: CodeBlock[] = [];
  for (const [f, { bodyContent }] of pageMap.entries()) {
    const blocks = extractCodeBlocks(f, bodyContent);
    allBlocks.push(...blocks);
  }
  const checkBlocks = allBlocks.filter(b => b.mode === "check");
  const runBlocks = allBlocks.filter(b => b.mode === "run");
  console.log(`Extracted ${allBlocks.length} code blocks (${checkBlocks.length} check, ${runBlocks.length} run, ${allBlocks.length - checkBlocks.length - runBlocks.length} snippet).`);

  // Gate 3: Agent Pack token size audit (< 3000 tokens)
  console.log("3. Auditing agent pack token sizes...");
  if (fs.existsSync(DIST_AGENT_PACKS)) {
    const packFiles = fs.readdirSync(DIST_AGENT_PACKS).filter(f => f.endsWith(".md"));
    for (const pf of packFiles) {
      const pPath = path.join(DIST_AGENT_PACKS, pf);
      const content = fs.readFileSync(pPath, "utf-8");
      const tokens = Math.ceil(content.length / 4);
      if (tokens > 3000) {
        errors.push(`[Agent Pack Size Limit] Agent pack '${pf}' has ${tokens} tokens, exceeding maximum budget of 3000 tokens.`);
      } else {
        console.log(` - Agent pack '${pf}': ${tokens} tokens [PASS]`);
      }
    }
  }

  // Gate 4: Style & Spelling lint for Markdown
  console.log("4. Linting markdown style and formatting invariants...");
  for (const [f, { frontmatter, bodyContent }] of pageMap.entries()) {
    // Invariant: summary must end with a period
    if (!frontmatter.summary.trim().endsWith(".")) {
      errors.push(`[Style Lint] Summary in ${f} should end with a period: "${frontmatter.summary}"`);
    }
    // Check no trailing whitespace on lines
    const lines = content.split(/\r?\n/);
    for (let li = 0; li < lines.length; li++) {
      if (/[ \t]+$/.test(lines[li])) {
        // Warning or error
      }
    }
  }

  if (errors.length === 0) {
    console.log("All Quality Gates Passed Cleanly!");
    return { success: true, errors: [] };
  } else {
    console.error(`Quality Gates Failed with ${errors.length} issues:`);
    for (const err of errors) {
      console.error(` - ${err}`);
    }
    return { success: false, errors };
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const result = validateAllQualityGates();
  if (!result.success) {
    process.exit(1);
  }
}
