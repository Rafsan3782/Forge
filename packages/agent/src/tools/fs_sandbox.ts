import fs from "node:fs/promises";
import path from "node:path";
import { ToolExecutionError } from "./errors.ts";
import { truncateOutput, isBinaryBuffer } from "./types.ts";

const DENIED_PATHS = [".git", ".forge/secrets", ".tools"];

export class ProjectSandbox {
  public readonly projectRoot: string;

  constructor(projectRoot: string) {
    this.projectRoot = path.resolve(projectRoot);
  }

  /**
   * Resolves a user-provided path within the project sandbox.
   * Throws ToolExecutionError if the path attempts to escape the project root or access denylisted paths.
   */
  public async resolveSafePath(userPath: string, checkExists = false): Promise<string> {
    if (!userPath || typeof userPath !== "string") {
      throw new ToolExecutionError(
        "INVALID_PATH",
        "Path must be a non-empty string",
        "Provide a valid relative path within the game project."
      );
    }

    const resolved = path.resolve(this.projectRoot, userPath);
    const relative = path.relative(this.projectRoot, resolved);

    if (relative.startsWith("..") || path.isAbsolute(relative)) {
      throw new ToolExecutionError(
        "SANDBOX_VIOLATION",
        `Access denied: path '${userPath}' escapes project root`,
        "Stay within the project root directory."
      );
    }

    const normalizedRelative = relative.replace(/\\/g, "/");
    for (const denied of DENIED_PATHS) {
      if (
        normalizedRelative === denied ||
        normalizedRelative.startsWith(denied + "/")
      ) {
        throw new ToolExecutionError(
          "ACCESS_DENIED",
          `Access to '${userPath}' is forbidden by sandbox policy`,
          "Do not access internal or secret files (.git, .forge/secrets, .tools)."
        );
      }
    }

    try {
      const realPath = await fs.realpath(resolved);
      const realRelative = path.relative(this.projectRoot, realPath);
      if (realRelative.startsWith("..") || path.isAbsolute(realRelative)) {
        throw new ToolExecutionError(
          "SANDBOX_VIOLATION",
          `Access denied: symlink '${userPath}' points outside project root`,
          "Symlinks pointing outside the project root are not allowed."
        );
      }
      for (const denied of DENIED_PATHS) {
        const norm = realRelative.replace(/\\/g, "/");
        if (norm === denied || norm.startsWith(denied + "/")) {
          throw new ToolExecutionError(
            "ACCESS_DENIED",
            `Access to '${userPath}' is forbidden by sandbox policy`,
            "Symlinks pointing to forbidden directories are not allowed."
          );
        }
      }
      return realPath;
    } catch (err: unknown) {
      const code = (err as { code?: string })?.code;
      if (code === "ENOENT") {
        if (checkExists) {
          throw new ToolExecutionError(
            "FILE_NOT_FOUND",
            `File or directory does not exist: '${userPath}'`,
            "Check path with fs.list or fs.glob before operating on it."
          );
        }
        let currentParent = path.dirname(resolved);
        while (currentParent.length >= this.projectRoot.length) {
          try {
            const parentReal = await fs.realpath(currentParent);
            const parentRel = path.relative(this.projectRoot, parentReal);
            if (parentRel.startsWith("..") || path.isAbsolute(parentRel)) {
              throw new ToolExecutionError(
                "SANDBOX_VIOLATION",
                `Access denied: path '${userPath}' parent escapes project root via symlink`,
                "Target parent directory must be within project root."
              );
            }
            break;
          } catch (e: unknown) {
            const parentErrCode = (e as { code?: string })?.code;
            if (parentErrCode === "ENOENT") {
              currentParent = path.dirname(currentParent);
              continue;
            }
            throw e;
          }
        }
        return resolved;
      }
      throw err;
    }
  }

  // --- File System Operations ---

  public async readFile(
    filePath: string,
    options?: { start_line?: number; end_line?: number }
  ): Promise<{ content: string; total_lines: number; start_line: number; end_line: number }> {
    const safePath = await this.resolveSafePath(filePath, true);
    const stat = await fs.stat(safePath);
    if (stat.isDirectory()) {
      throw new ToolExecutionError(
        "IS_DIRECTORY",
        `Path '${filePath}' is a directory, not a file`,
        "Use fs.list to view directory contents."
      );
    }

    const rawBuffer = await fs.readFile(safePath);
    if (isBinaryBuffer(rawBuffer)) {
      throw new ToolExecutionError(
        "BINARY_FILE",
        `File '${filePath}' appears to be binary and cannot be read as text`,
        "Binary files cannot be read with fs.read."
      );
    }

    const fullText = rawBuffer.toString("utf8");
    const lines = fullText.split(/\r?\n/);
    const totalLines = lines.length;

    let start = options?.start_line ?? 1;
    let end = options?.end_line ?? totalLines;

    if (start < 1) start = 1;
    if (end > totalLines) end = totalLines;
    if (start > end) {
      throw new ToolExecutionError(
        "INVALID_LINE_RANGE",
        `start_line (${start}) cannot be greater than end_line (${end})`,
        "Provide a valid 1-indexed line range."
      );
    }

    const sliced = lines.slice(start - 1, end).join("\n");
    const formatted = truncateOutput(sliced, 10000, `fs.read with smaller line range for '${filePath}'`);

    return {
      content: formatted,
      total_lines: totalLines,
      start_line: start,
      end_line: end,
    };
  }

  public async writeFile(
    filePath: string,
    content: string
  ): Promise<{ success: boolean; bytes_written: number; path: string }> {
    const safePath = await this.resolveSafePath(filePath, false);
    await fs.mkdir(path.dirname(safePath), { recursive: true });
    const buffer = Buffer.from(content, "utf8");
    await fs.writeFile(safePath, buffer);
    return {
      success: true,
      bytes_written: buffer.length,
      path: path.relative(this.projectRoot, safePath).replace(/\\/g, "/"),
    };
  }

  public async editFile(
    filePath: string,
    searchText: string,
    replaceText: string
  ): Promise<{ success: boolean; diff: string; path: string }> {
    if (!searchText) {
      throw new ToolExecutionError(
        "INVALID_SEARCH_TEXT",
        "search_text must not be empty",
        "Provide the exact string block to replace."
      );
    }

    const safePath = await this.resolveSafePath(filePath, true);
    const rawBuffer = await fs.readFile(safePath);
    if (isBinaryBuffer(rawBuffer)) {
      throw new ToolExecutionError(
        "BINARY_FILE",
        `File '${filePath}' appears to be binary and cannot be edited as text`,
        "Binary files cannot be edited with fs.edit."
      );
    }

    const content = rawBuffer.toString("utf8");
    const occurrences = content.split(searchText).length - 1;

    if (occurrences === 0) {
      throw new ToolExecutionError(
        "SEARCH_NOT_FOUND",
        `search_text was not found in '${filePath}'`,
        "Ensure search_text matches file contents exactly, including whitespace."
      );
    }
    if (occurrences > 1) {
      throw new ToolExecutionError(
        "AMBIGUOUS_MATCH",
        `search_text matched ${occurrences} times in '${filePath}'. fs.edit requires an exact single match`,
        "Provide more surrounding context lines in search_text to ensure a unique match."
      );
    }

    const newContent = content.replace(searchText, replaceText);
    await fs.writeFile(safePath, Buffer.from(newContent, "utf8"));

    const relPath = path.relative(this.projectRoot, safePath).replace(/\\/g, "/");
    const diff = `--- a/${relPath}\n+++ b/${relPath}\n@@ -1 +1 @@\n-${searchText.split("\n").join("\n-")}\n+${replaceText.split("\n").join("\n+")}`;

    return {
      success: true,
      diff,
      path: relPath,
    };
  }

  public async listDirectory(
    dirPath = "."
  ): Promise<Array<{ name: string; is_dir: boolean; size_bytes?: number }>> {
    const safePath = await this.resolveSafePath(dirPath, true);
    const entries = await fs.readdir(safePath, { withFileTypes: true });
    const results: Array<{ name: string; is_dir: boolean; size_bytes?: number }> = [];

    for (const entry of entries) {
      const relToRoot = path.relative(
        this.projectRoot,
        path.join(safePath, entry.name)
      ).replace(/\\/g, "/");

      let denied = false;
      for (const d of DENIED_PATHS) {
        if (relToRoot === d || relToRoot.startsWith(d + "/")) {
          denied = true;
          break;
        }
      }
      if (denied) continue;

      let isDir = entry.isDirectory();
      let sizeBytes: number | undefined;
      try {
        const stat = await fs.stat(path.join(safePath, entry.name));
        isDir = stat.isDirectory();
        if (!isDir) sizeBytes = stat.size;
      } catch {
        // stat failed
      }

      results.push({
        name: entry.name,
        is_dir: isDir,
        size_bytes: sizeBytes,
      });
    }

    return results;
  }

  public async glob(
    pattern: string,
    baseDir = "."
  ): Promise<string[]> {
    const safeBase = await this.resolveSafePath(baseDir, true);
    const matches: string[] = [];
    const regexPattern = globToRegex(pattern);

    async function walk(currentDir: string, rootDir: string, denylist: string[]) {
      const entries = await fs.readdir(currentDir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name);
        const relToRoot = path.relative(rootDir, fullPath).replace(/\\/g, "/");

        let denied = false;
        for (const d of denylist) {
          if (relToRoot === d || relToRoot.startsWith(d + "/")) {
            denied = true;
            break;
          }
        }
        if (denied) continue;

        const relToBase = path.relative(safeBase, fullPath).replace(/\\/g, "/");
        if (regexPattern.test(relToBase)) {
          matches.push(relToBase);
        }

        if (entry.isDirectory()) {
          await walk(fullPath, rootDir, denylist);
        }
      }
    }

    await walk(safeBase, this.projectRoot, DENIED_PATHS);
    return matches;
  }

  public async search(
    query: string,
    options?: { path?: string; context_lines?: number; is_regex?: boolean; max_results?: number }
  ): Promise<Array<{ file: string; line_number: number; line: string; context: string[] }>> {
    const searchBase = options?.path ? await this.resolveSafePath(options.path, true) : this.projectRoot;
    const contextCount = options?.context_lines ?? 2;
    const maxResults = options?.max_results ?? 100;
    const isRegex = options?.is_regex ?? false;

    let regex: RegExp;
    try {
      regex = isRegex ? new RegExp(query, "g") : new RegExp(escapeRegex(query), "g");
    } catch (e: unknown) {
      throw new ToolExecutionError(
        "INVALID_REGEX",
        `Invalid regular expression: ${(e as Error).message}`,
        "Provide a valid regular expression or set is_regex to false."
      );
    }

    const results: Array<{ file: string; line_number: number; line: string; context: string[] }> = [];

    const walk = async (currentDir: string) => {
      if (results.length >= maxResults) return;
      const entries = await fs.readdir(currentDir, { withFileTypes: true });

      for (const entry of entries) {
        if (results.length >= maxResults) break;
        const fullPath = path.join(currentDir, entry.name);
        const relToRoot = path.relative(this.projectRoot, fullPath).replace(/\\/g, "/");

        let isDenied = false;
        for (const d of DENIED_PATHS) {
          if (relToRoot === d || relToRoot.startsWith(d + "/")) {
            isDenied = true;
            break;
          }
        }
        if (isDenied) continue;

        if (entry.isDirectory()) {
          await walk(fullPath);
        } else if (entry.isFile()) {
          try {
            const buf = await fs.readFile(fullPath);
            if (isBinaryBuffer(buf)) continue;

            const text = buf.toString("utf8");
            const lines = text.split(/\r?\n/);
            for (let i = 0; i < lines.length; i++) {
              if (results.length >= maxResults) break;
              const line = lines[i];
              if (regex.test(line)) {
                regex.lastIndex = 0;
                const startCtx = Math.max(0, i - contextCount);
                const endCtx = Math.min(lines.length - 1, i + contextCount);
                const context: string[] = [];
                for (let c = startCtx; c <= endCtx; c++) {
                  context.push(lines[c]);
                }
                results.push({
                  file: relToRoot,
                  line_number: i + 1,
                  line,
                  context,
                });
              }
            }
          } catch {
            // Ignore file read errors
          }
        }
      }
    };

    const stat = await fs.stat(searchBase);
    if (stat.isFile()) {
      const buf = await fs.readFile(searchBase);
      if (!isBinaryBuffer(buf)) {
        const text = buf.toString("utf8");
        const lines = text.split(/\r?\n/);
        const relToRoot = path.relative(this.projectRoot, searchBase).replace(/\\/g, "/");
        for (let i = 0; i < lines.length; i++) {
          if (results.length >= maxResults) break;
          const line = lines[i];
          if (regex.test(line)) {
            regex.lastIndex = 0;
            const startCtx = Math.max(0, i - contextCount);
            const endCtx = Math.min(lines.length - 1, i + contextCount);
            const context: string[] = [];
            for (let c = startCtx; c <= endCtx; c++) {
              context.push(lines[c]);
            }
            results.push({
              file: relToRoot,
              line_number: i + 1,
              line,
              context,
            });
          }
        }
      }
    } else {
      await walk(searchBase);
    }

    return results;
  }

  public async move(
    sourcePath: string,
    targetPath: string
  ): Promise<{ success: boolean; from: string; to: string }> {
    const safeSource = await this.resolveSafePath(sourcePath, true);
    const safeTarget = await this.resolveSafePath(targetPath, false);

    await fs.mkdir(path.dirname(safeTarget), { recursive: true });
    await fs.rename(safeSource, safeTarget);

    return {
      success: true,
      from: path.relative(this.projectRoot, safeSource).replace(/\\/g, "/"),
      to: path.relative(this.projectRoot, safeTarget).replace(/\\/g, "/"),
    };
  }

  public async delete(
    targetPath: string
  ): Promise<{ success: boolean; path: string }> {
    const safePath = await this.resolveSafePath(targetPath, true);
    const stat = await fs.stat(safePath);
    if (stat.isDirectory()) {
      await fs.rm(safePath, { recursive: true, force: true });
    } else {
      await fs.unlink(safePath);
    }

    return {
      success: true,
      path: path.relative(this.projectRoot, safePath).replace(/\\/g, "/"),
    };
  }
}

function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function globToRegex(glob: string): RegExp {
  // Normalize forward slashes
  const normalized = glob.replace(/\\/g, "/");
  let reStr = "^";
  let i = 0;
  while (i < normalized.length) {
    const c = normalized[i];
    if (c === "*" && normalized[i + 1] === "*") {
      if (normalized[i + 2] === "/") {
        reStr += "(?:.*/)?";
        i += 3;
      } else {
        reStr += ".*";
        i += 2;
      }
    } else if (c === "*") {
      reStr += "[^/]*";
      i++;
    } else if (c === "?") {
      reStr += "[^/]";
      i++;
    } else if (/[.+^${}()|[\]\\]/.test(c)) {
      reStr += "\\" + c;
      i++;
    } else {
      reStr += c;
      i++;
    }
  }
  reStr += "$";
  return new RegExp(reStr);
}
