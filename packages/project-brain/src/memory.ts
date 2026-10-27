export const DEFAULT_MEMORY_CAP_BYTES = 10 * 1024;

export function compactMemory(
  content: string,
  maxBytes: number = DEFAULT_MEMORY_CAP_BYTES
): string {
  const byteLength = Buffer.byteLength(content, "utf8");
  if (byteLength <= maxBytes) {
    return content;
  }

  const lines = content.split(/\r?\n/);
  const entries: string[] = [];
  const summaryHeader = "# Agent Memory\n";
  let existingArchiveSummary = "";

  let inArchive = false;
  const archiveLines: string[] = [];


  for (const line of lines) {
    if (line.startsWith("# Agent Memory")) {
      continue;
    }
    if (line.includes("<!-- ARCHIVE_SUMMARY_START -->")) {
      inArchive = true;
      continue;
    }
    if (line.includes("<!-- ARCHIVE_SUMMARY_END -->")) {
      inArchive = false;
      existingArchiveSummary = archiveLines.join("\n").trim();
      continue;
    }

    if (inArchive) {
      archiveLines.push(line);
      continue;
    }

    const trimmed = line.trim();
    if (trimmed.length > 0) {
      entries.push(line);
    }
  }

  const keepCount = Math.max(3, Math.floor(entries.length * 0.4));
  const toCompact = entries.slice(0, entries.length - keepCount);
  const toKeep = entries.slice(entries.length - keepCount);

  const newSummaryBullets: string[] = [];
  for (const item of toCompact) {
    const clean = item.replace(/^[-*#\s\d:_/()[\]-]+/, "").trim();
    if (clean.length > 0) {
      const short = clean.length > 80 ? clean.slice(0, 77) + "..." : clean;
      newSummaryBullets.push(`- ${short}`);
    }
  }

  let combinedArchive = existingArchiveSummary;
  if (newSummaryBullets.length > 0) {
    if (combinedArchive) {
      combinedArchive += "\n" + newSummaryBullets.join("\n");
    } else {
      combinedArchive = newSummaryBullets.join("\n");
    }
  }

  const resultParts = [
    summaryHeader.trim(),
    "",
    "<!-- ARCHIVE_SUMMARY_START -->",
    "## Compacted Knowledge Summary",
    combinedArchive.trim(),
    "<!-- ARCHIVE_SUMMARY_END -->",
    "",
    "## Active Notes",
    toKeep.join("\n").trim(),
    "",
  ];

  return resultParts.join("\n");
}

export function appendMemoryNote(
  existingContent: string,
  note: string,
  maxBytes: number = DEFAULT_MEMORY_CAP_BYTES
): string {
  const timestamp = new Date().toISOString().replace("T", " ").slice(0, 19);
  const noteLine = `- [${timestamp}]: ${note.trim()}`;

  const updated = existingContent.trim()
    ? `${existingContent.trim()}\n${noteLine}\n`
    : `# Agent Memory\n\n## Active Notes\n${noteLine}\n`;

  return compactMemory(updated, maxBytes);
}

