import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { generateBridgeTools, BRIDGE_TOOLS } from "./generate-tools.js";
import { generateComponents, COMPONENTS } from "./generate-components.js";
import { generateEventsAndErrors, EVENTS, ERROR_CATALOG } from "./generate-events-errors.js";
import { generateScriptApi, SCRIPT_MODULES } from "./generate-script-api.js";
import { generateCli, CLI_COMMANDS } from "./generate-cli.js";
import { generateAgentPacks, AGENT_PACK_TOPICS } from "./generate-agent-packs.js";
import { parseFrontmatter, estimateTokens, ParsedDocPage } from "./schema.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "../..");
const DOCS_SRC = path.join(REPO_ROOT, "docs/src");
const DOCS_DIST = path.join(REPO_ROOT, "docs/dist");
const HUMAN_BUNDLE_DIR = path.join(DOCS_DIST, "human-bundle");

// Ensure distribution directories exist
fs.mkdirSync(DOCS_DIST, { recursive: true });
fs.mkdirSync(HUMAN_BUNDLE_DIR, { recursive: true });

export function collectAllDocFiles(dir: string): string[] {
  const files: string[] = [];
  const entries = fs.readdirSync(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectAllDocFiles(fullPath));
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      files.push(fullPath);
    }
  }

  return files;
}

export interface NavNode {
  title: string;
  id?: string;
  path?: string;
  children?: NavNode[];
}

export function buildNavTree(baseDir: string): NavNode[] {
  const rootNodes: NavNode[] = [];

  const sections = [
    { folder: "guide", label: "Guides" },
    { folder: "concepts", label: "Concepts" },
    { folder: "reference", label: "Reference" },
    { folder: "cookbook", label: "Cookbook Recipes" },
    { folder: "agents", label: "Agent Manual" }
  ];

  for (const sec of sections) {
    const secDir = path.join(baseDir, sec.folder);
    if (!fs.existsSync(secDir)) continue;

    const files = collectAllDocFiles(secDir);
    const children: NavNode[] = files.map(f => {
      const rel = path.relative(baseDir, f).replace(/\\/g, "/");
      const content = fs.readFileSync(f, "utf-8");
      try {
        const { frontmatter } = parseFrontmatter(content, f);
        return {
          title: frontmatter.title,
          id: frontmatter.id,
          path: rel
        };
      } catch {
        return {
          title: path.basename(f, ".md"),
          path: rel
        };
      }
    });

    rootNodes.push({
      title: sec.label,
      children
    });
  }

  // Top level changelog if present
  const changelogPath = path.join(baseDir, "changelog.md");
  if (fs.existsSync(changelogPath)) {
    rootNodes.push({
      title: "Changelog",
      id: "changelog",
      path: "changelog.md"
    });
  }

  return rootNodes;
}

export async function buildDocumentation() {
  console.log("=== Building Forge Documentation System ===");

  // 1. Run all Generators
  console.log("1. Running generators...");
  generateBridgeTools();
  generateComponents();
  generateEventsAndErrors();
  generateScriptApi();
  generateCli();
  const agentPacks = generateAgentPacks();

  // 2. Scan and parse all docs in docs/src
  console.log("2. Scanning and validating docs in docs/src...");
  const allDocPaths = collectAllDocFiles(DOCS_SRC);
  const parsedPages: ParsedDocPage[] = [];
  const pageIdMap = new Map<string, ParsedDocPage>();

  for (const docPath of allDocPaths) {
    const rawContent = fs.readFileSync(docPath, "utf-8");
    const { frontmatter, bodyContent } = parseFrontmatter(rawContent, docPath);

    if (pageIdMap.has(frontmatter.id)) {
      throw new Error(`Duplicate doc ID found: '${frontmatter.id}' in ${docPath} (already registered by ${pageIdMap.get(frontmatter.id)?.filePath})`);
    }

    const tokenCount = estimateTokens(rawContent);
    const parsedPage: ParsedDocPage = {
      filePath: docPath,
      frontmatter,
      rawContent,
      bodyContent,
      tokenCount
    };

    parsedPages.push(parsedPage);
    pageIdMap.set(frontmatter.id, parsedPage);
  }

  console.log(`Validated ${parsedPages.length} documentation pages against DocsPageSchema.`);

  // 3. Build Human Bundle & Nav Tree
  console.log("3. Emitting human bundle to docs/dist/human-bundle...");
  for (const page of parsedPages) {
    const rel = path.relative(DOCS_SRC, page.filePath);
    const dest = path.join(HUMAN_BUNDLE_DIR, rel);
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    fs.writeFileSync(dest, page.rawContent, "utf-8");
  }

  const navTree = buildNavTree(DOCS_SRC);
  fs.writeFileSync(path.join(DOCS_DIST, "nav.json"), JSON.stringify(navTree, null, 2), "utf-8");
  fs.writeFileSync(path.join(HUMAN_BUNDLE_DIR, "nav.json"), JSON.stringify(navTree, null, 2), "utf-8");

  // 4. Build llms.txt & llms-full.txt
  console.log("4. Emitting llms.txt and llms-full.txt...");
  const llmsHeader = `# Forge Engine Documentation
> Forge is an AI-native game engine and Windows Studio IDE for real-time collaboration between human creators and autonomous agent workflows.

## Core Documentation Pages
`;

  const llmsEntries = parsedPages
    .sort((a, b) => a.frontmatter.title.localeCompare(b.frontmatter.title))
    .map(p => {
      const rel = path.relative(DOCS_SRC, p.filePath).replace(/\\/g, "/");
      return `- [${p.frontmatter.title}](human-bundle/${rel}): ${p.frontmatter.summary}`;
    })
    .join("\n");

  const llmsAgentPacksSection = `\n\n## Compact Agent Packs (Under 3000 Tokens Each)\n` +
    agentPacks.map(ap => `- [${ap.name}](agent-packs/${ap.name}.md): Self-contained brief on ${ap.name} (${ap.tokenCount} tokens)`).join("\n");

  const llmsTxtContent = `${llmsHeader}\n${llmsEntries}${llmsAgentPacksSection}\n`;
  fs.writeFileSync(path.join(DOCS_DIST, "llms.txt"), llmsTxtContent, "utf-8");

  // llms-full.txt: Everything concatenated with delimiters
  let llmsFullContent = `# Forge Complete Documentation (llms-full.txt)\n\n`;
  for (const page of parsedPages) {
    const rel = path.relative(DOCS_SRC, page.filePath).replace(/\\/g, "/");
    llmsFullContent += `\n\n<!-- PAGE: ${rel} (ID: ${page.frontmatter.id}) -->\n\n${page.rawContent}\n\n---`;
  }
  fs.writeFileSync(path.join(DOCS_DIST, "llms-full.txt"), llmsFullContent, "utf-8");

  // 5. Build Machine-Readable index.json
  console.log("5. Emitting machine-readable index.json...");
  const indexEntries = parsedPages.map(p => ({
    id: p.frontmatter.id,
    title: p.frontmatter.title,
    summary: p.frontmatter.summary,
    tags: p.frontmatter.tags,
    audience: p.frontmatter.audience,
    path: path.relative(DOCS_SRC, p.filePath).replace(/\\/g, "/"),
    token_count: p.tokenCount,
    related_ids: p.frontmatter.related_ids
  }));

  const fullIndex = {
    generated_at: new Date().toISOString(),
    engine_version: "0.1.0",
    total_pages: parsedPages.length,
    pages: indexEntries,
    tools: BRIDGE_TOOLS.map(t => ({ name: t.name, category: t.category, purpose: t.purpose })),
    components: COMPONENTS.map(c => ({ name: c.name, category: c.category, purpose: c.purpose })),
    events: EVENTS.map(e => ({ name: e.name, source: e.source, description: e.description })),
    error_codes: ERROR_CATALOG.map(ec => ({ code: ec.code, meaning: ec.meaning, fix: ec.fix })),
    agent_packs: agentPacks.map(ap => ({ name: ap.name, token_count: ap.tokenCount }))
  };

  fs.writeFileSync(path.join(DOCS_DIST, "index.json"), JSON.stringify(fullIndex, null, 2), "utf-8");

  // 6. Build Lightweight Search Index (MiniSearch-compatible JSON)
  console.log("6. Emitting search-index.json for Studio IDE and prompt-27 tools...");
  const searchDocs = parsedPages.map((p, idx) => ({
    id: p.frontmatter.id,
    numericId: idx + 1,
    title: p.frontmatter.title,
    summary: p.frontmatter.summary,
    tags: p.frontmatter.tags.join(" "),
    content: p.bodyContent.replace(/```[\s\S]*?```/g, " ").replace(/[#*`_[\]()-]/g, " ").slice(0, 4000),
    path: path.relative(DOCS_SRC, p.filePath).replace(/\\/g, "/")
  }));

  const searchIndex = {
    version: "1.0",
    fields: ["title", "summary", "tags", "content"],
    documents: searchDocs
  };
  fs.writeFileSync(path.join(DOCS_DIST, "search-index.json"), JSON.stringify(searchIndex, null, 2), "utf-8");

  // 7. Zip Bundle creation (using Node's built-in zlib / zip stream or standard zip package)
  console.log("7. Emitting docs-bundle.zip for website deployment...");
  await createZipBundle(DOCS_DIST, path.join(DOCS_DIST, "docs-bundle.zip"));

  console.log("=== Documentation Build Finished Successfully! ===");
}

async function createZipBundle(sourceDir: string, zipFilePath: string) {
  // Use a self-contained pure JS minimal zip writer to avoid external dependencies
  const filesToZip: Array<{ relPath: string; data: Buffer }> = [];

  function addFiles(currentDir: string, baseDir: string) {
    const entries = fs.readdirSync(currentDir, { withFileTypes: true });
    for (const entry of entries) {
      const full = path.join(currentDir, entry.name);
      if (entry.name === "docs-bundle.zip") continue;

      if (entry.isDirectory()) {
        addFiles(full, baseDir);
      } else if (entry.isFile()) {
        const rel = path.relative(baseDir, full).replace(/\\/g, "/");
        filesToZip.push({
          relPath: rel,
          data: fs.readFileSync(full)
        });
      }
    }
  }

  addFiles(sourceDir, sourceDir);

  const zipBuffer = buildPureZip(filesToZip);
  fs.writeFileSync(zipFilePath, zipBuffer);
  console.log(`Created ${zipFilePath} (${(zipBuffer.length / 1024).toFixed(1)} KB) containing ${filesToZip.length} files.`);
}

function buildPureZip(files: Array<{ relPath: string; data: Buffer }>): Buffer {
  // Minimal PKZip implementation with STORE (uncompressed) or DEFLATE via zlib
  import("node:zlib");
  const localHeaders: Buffer[] = [];
  const centralDirs: Buffer[] = [];
  let offset = 0;

  for (const file of files) {
    const fileNameBuf = Buffer.from(file.relPath, "utf-8");
    const crc = computeCrc32(file.data);
    const size = file.data.length;

    // Local file header (30 bytes + filename)
    const localHeader = Buffer.alloc(30 + fileNameBuf.length);
    localHeader.writeUInt32LE(0x04034b50, 0); // Signature
    localHeader.writeUInt16LE(20, 4); // Version needed (2.0)
    localHeader.writeUInt16LE(0, 6);  // Flags
    localHeader.writeUInt16LE(0, 8);  // Compression (0 = STORE)
    localHeader.writeUInt16LE(0, 10); // Mod time
    localHeader.writeUInt16LE(0, 12); // Mod date
    localHeader.writeUInt32LE(crc, 14); // CRC-32
    localHeader.writeUInt32LE(size, 18); // Compressed size
    localHeader.writeUInt32LE(size, 22); // Uncompressed size
    localHeader.writeUInt16LE(fileNameBuf.length, 26); // Filename length
    localHeader.writeUInt16LE(0, 28); // Extra field length
    fileNameBuf.copy(localHeader, 30);

    // Central directory header (46 bytes + filename)
    const centralDir = Buffer.alloc(46 + fileNameBuf.length);
    centralDir.writeUInt32LE(0x02014b50, 0); // Signature
    centralDir.writeUInt16LE(20, 4); // Version made by
    centralDir.writeUInt16LE(20, 6); // Version needed
    centralDir.writeUInt16LE(0, 8);  // Flags
    centralDir.writeUInt16LE(0, 10); // Compression
    centralDir.writeUInt16LE(0, 12); // Mod time
    centralDir.writeUInt16LE(0, 14); // Mod date
    centralDir.writeUInt32LE(crc, 16); // CRC-32
    centralDir.writeUInt32LE(size, 20); // Compressed size
    centralDir.writeUInt32LE(size, 24); // Uncompressed size
    centralDir.writeUInt16LE(fileNameBuf.length, 28); // Filename length
    centralDir.writeUInt16LE(0, 30); // Extra field length
    centralDir.writeUInt16LE(0, 32); // File comment length
    centralDir.writeUInt16LE(0, 34); // Disk number start
    centralDir.writeUInt16LE(0, 36); // Internal file attributes
    centralDir.writeUInt32LE(0, 38); // External file attributes
    centralDir.writeUInt32LE(offset, 42); // Relative offset of local header
    fileNameBuf.copy(centralDir, 46);

    localHeaders.push(localHeader, file.data);
    centralDirs.push(centralDir);
    offset += localHeader.length + file.data.length;
  }

  const centralDirOffset = offset;
  let centralDirSize = 0;
  for (const cd of centralDirs) {
    centralDirSize += cd.length;
  }

  // End of central directory record (22 bytes)
  const eocd = Buffer.alloc(22);
  eocd.writeUInt32LE(0x06054b50, 0); // Signature
  eocd.writeUInt16LE(0, 4); // Number of this disk
  eocd.writeUInt16LE(0, 6); // Disk where central directory starts
  eocd.writeUInt16LE(files.length, 8); // Number of central directory records on this disk
  eocd.writeUInt16LE(files.length, 10); // Total number of central directory records
  eocd.writeUInt32LE(centralDirSize, 12); // Size of central directory
  eocd.writeUInt32LE(centralDirOffset, 16); // Offset of start of central directory
  eocd.writeUInt16LE(0, 20); // Comment length

  return Buffer.concat([...localHeaders, ...centralDirs, eocd]);
}

function computeCrc32(buf: Buffer): number {
  let crc = -1;
  for (let i = 0; i < buf.length; i++) {
    const byte = buf[i];
    crc = (crc >>> 8) ^ CRC_TABLE[(crc ^ byte) & 0xff];
  }
  return (crc ^ -1) >>> 0;
}

const CRC_TABLE = new Uint32Array(256);
for (let i = 0; i < 256; i++) {
  let c = i;
  for (let k = 0; k < 8; k++) {
    c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  }
  CRC_TABLE[i] = c >>> 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  buildDocumentation().catch((err) => {
    console.error("Documentation build failed:", err);
    process.exit(1);
  });
}
