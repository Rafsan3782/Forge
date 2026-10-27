import type { Section, MergeConflict, MergeResult } from "./types.ts";

export function normalizeSectionId(heading: string): string {
  const anchorMatch = heading.match(/<!--\s*(?:id|section):\s*([a-zA-Z0-9_-]+)\s*-->/);
  if (anchorMatch) {
    return anchorMatch[1].toLowerCase();
  }

  return heading
    .replace(/^#+\s*/, "")
    .replace(/<!--.*?-->/g, "")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

export function parseSections(markdown: string): Section[] {
  const lines = markdown.split(/\r?\n/);
  const sections: Section[] = [];

  let currentHeading = "";
  let currentLevel = 0;
  let currentId = "preamble";
  let currentLines: string[] = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const headingMatch = line.match(/^(#{1,6})\s+(.*)$/);

    if (headingMatch) {
      if (currentHeading !== "" || currentLines.length > 0) {
        sections.push({
          id: currentId,
          heading: currentHeading,
          level: currentLevel,
          content: currentLines.join("\n"),
        });
      }

      currentLevel = headingMatch[1].length;
      currentHeading = headingMatch[2].trim();
      currentId = normalizeSectionId(currentHeading);
      currentLines = [];
    } else {
      currentLines.push(line);
    }
  }

  sections.push({
    id: currentId,
    heading: currentHeading,
    level: currentLevel,
    content: currentLines.join("\n"),
  });

  return sections;
}

export function serializeSections(sections: Section[]): string {
  const parts: string[] = [];

  for (const section of sections) {
    if (section.heading) {
      const hashes = "#".repeat(section.level || 1);
      parts.push(`${hashes} ${section.heading}`);
    }
    if (section.content !== "") {
      parts.push(section.content);
    }
  }

  return parts.join("\n\n").trimEnd() + "\n";
}

export function patchSection(
  markdown: string,
  sectionId: string,
  newContent: string
): string {
  const sections = parseSections(markdown);
  const targetId = sectionId.toLowerCase();
  let found = false;

  for (const s of sections) {
    if (s.id.toLowerCase() === targetId) {
      s.content = newContent.trim();
      found = true;
      break;
    }
  }

  if (!found) {
    sections.push({
      id: targetId,
      heading: sectionId
        .split("-")
        .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
        .join(" "),
      level: 2,
      content: newContent.trim(),
    });
  }

  return serializeSections(sections);
}

export function mergeSectionsThreeWay(
  baseMarkdown: string,
  userMarkdown: string,
  agentMarkdown: string
): MergeResult {
  const baseSections = new Map(parseSections(baseMarkdown).map((s) => [s.id, s]));
  const userSections = new Map(parseSections(userMarkdown).map((s) => [s.id, s]));
  const agentSections = new Map(parseSections(agentMarkdown).map((s) => [s.id, s]));

  const allIds = new Set<string>();
  for (const id of baseSections.keys()) allIds.add(id);
  for (const id of userSections.keys()) allIds.add(id);
  for (const id of agentSections.keys()) allIds.add(id);

  const mergedSections: Section[] = [];
  const conflicts: MergeConflict[] = [];

  for (const id of allIds) {
    const baseSec = baseSections.get(id);
    const userSec = userSections.get(id);
    const agentSec = agentSections.get(id);

    const baseContent = baseSec ? baseSec.content.trim() : "";
    const userContent = userSec ? userSec.content.trim() : "";
    const agentContent = agentSec ? agentSec.content.trim() : "";

    const templateSec = userSec || agentSec || baseSec;
    if (!templateSec) continue;

    if (agentContent === baseContent) {
      if (userSec) {
        mergedSections.push(userSec);
      }
      continue;
    }

    if (userContent === baseContent) {
      if (agentSec) {
        mergedSections.push({
          ...templateSec,
          content: agentContent,
        });
      }
      continue;
    }

    if (userContent === agentContent) {
      if (userSec) {
        mergedSections.push(userSec);
      }
      continue;
    }

    conflicts.push({
      sectionId: id,
      baseContent,
      userContent,
      agentContent,
    });

    mergedSections.push({
      ...templateSec,
      content: `<<<<<<< USER\n${userContent}\n=======\n${agentContent}\n>>>>>>> AGENT`,
    });
  }

  return {
    mergedContent: serializeSections(mergedSections),
    conflicts,
  };
}
