import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import os from "node:os";
import { ProjectSandbox } from "./fs_sandbox.ts";
import { createFsTools } from "./fs_tools.ts";
import { ToolRegistry } from "./registry.ts";
import { ToolExecutionError } from "./errors.ts";

test("ProjectSandbox: rejects path escaping project root (..\\..\\secret.txt)", async () => {
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "forge-sandbox-escape-"));
  try {
    const sandbox = new ProjectSandbox(tmpDir);

    await assert.rejects(
      async () => {
        await sandbox.resolveSafePath("..\\..\\secret.txt");
      },
      (err: unknown) => {
        return err instanceof ToolExecutionError && err.code === "SANDBOX_VIOLATION";
      }
    );

    await assert.rejects(
      async () => {
        await sandbox.resolveSafePath("../../secret.txt");
      },
      (err: unknown) => {
        return err instanceof ToolExecutionError && err.code === "SANDBOX_VIOLATION";
      }
    );
  } finally {
    await fs.rm(tmpDir, { recursive: true, force: true });
  }
});

test("ProjectSandbox: rejects symlink escaping project root", async () => {
  const outerDir = await fs.mkdtemp(path.join(os.tmpdir(), "forge-outer-"));
  const projectDir = path.join(outerDir, "project");
  const secretFile = path.join(outerDir, "secret.txt");

  await fs.mkdir(projectDir);
  await fs.writeFile(secretFile, "super_secret_token", "utf8");

  const symlinkPath = path.join(projectDir, "leaked_link.txt");
  try {
    await fs.symlink(secretFile, symlinkPath);
  } catch {
    return;
  }

  try {
    const sandbox = new ProjectSandbox(projectDir);

    await assert.rejects(
      async () => {
        await sandbox.readFile("leaked_link.txt");
      },
      (err: unknown) => {
        return err instanceof ToolExecutionError && err.code === "SANDBOX_VIOLATION";
      }
    );
  } finally {
    await fs.rm(outerDir, { recursive: true, force: true });
  }
});

test("ProjectSandbox: denylists .git, .forge/secrets, and .tools", async () => {
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "forge-denylist-"));
  try {
    const sandbox = new ProjectSandbox(tmpDir);

    await assert.rejects(
      async () => {
        await sandbox.resolveSafePath(".git/config");
      },
      (err: unknown) => err instanceof ToolExecutionError && err.code === "ACCESS_DENIED"
    );

    await assert.rejects(
      async () => {
        await sandbox.resolveSafePath(".forge/secrets/key.txt");
      },
      (err: unknown) => err instanceof ToolExecutionError && err.code === "ACCESS_DENIED"
    );

    await assert.rejects(
      async () => {
        await sandbox.resolveSafePath(".tools/bin/tool");
      },
      (err: unknown) => err instanceof ToolExecutionError && err.code === "ACCESS_DENIED"
    );
  } finally {
    await fs.rm(tmpDir, { recursive: true, force: true });
  }
});

test("fs.read and fs.write operations with line slicing", async () => {
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "forge-fs-rw-"));
  try {
    const sandbox = new ProjectSandbox(tmpDir);
    const registry = new ToolRegistry();
    registry.registerTools(createFsTools(sandbox));

    const writeRes = await registry.executeTool("fs.write", {
      path: "src/main.ts",
      content: "line 1\nline 2\nline 3\nline 4\nline 5",
    });
    assert.equal(writeRes.is_error, false);

    const readFull = await registry.executeTool("fs.read", {
      path: "src/main.ts",
    });
    assert.equal(readFull.is_error, false);
    const fullData = readFull.content as { content: string; total_lines: number };
    assert.equal(fullData.total_lines, 5);
    assert.ok(fullData.content.includes("line 1"));

    const readSlice = await registry.executeTool("fs.read", {
      path: "src/main.ts",
      start_line: 2,
      end_line: 4,
    });
    assert.equal(readSlice.is_error, false);
    const sliceData = readSlice.content as { content: string; start_line: number; end_line: number };
    assert.equal(sliceData.content, "line 2\nline 3\nline 4");
    assert.equal(sliceData.start_line, 2);
    assert.equal(sliceData.end_line, 4);
  } finally {
    await fs.rm(tmpDir, { recursive: true, force: true });
  }
});

test("fs.edit fails when search text matches zero or several times", async () => {
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "forge-fs-edit-"));
  try {
    const sandbox = new ProjectSandbox(tmpDir);
    const registry = new ToolRegistry();
    registry.registerTools(createFsTools(sandbox));

    await registry.executeTool("fs.write", {
      path: "test.txt",
      content: "apple\nbanana\napple\norange",
    });

    const zeroMatch = await registry.executeTool("fs.edit", {
      path: "test.txt",
      search_text: "pineapple",
      replace_text: "grape",
    });
    assert.equal(zeroMatch.is_error, true);
    assert.equal((zeroMatch.content as { code: string }).code, "SEARCH_NOT_FOUND");

    const multiMatch = await registry.executeTool("fs.edit", {
      path: "test.txt",
      search_text: "apple",
      replace_text: "grape",
    });
    assert.equal(multiMatch.is_error, true);
    assert.equal((multiMatch.content as { code: string }).code, "AMBIGUOUS_MATCH");

    const singleMatch = await registry.executeTool("fs.edit", {
      path: "test.txt",
      search_text: "banana",
      replace_text: "mango",
    });
    assert.equal(singleMatch.is_error, false);
    const diffData = singleMatch.content as { diff: string };
    assert.ok(diffData.diff.includes("-banana"));
    assert.ok(diffData.diff.includes("+mango"));

    const verifyRead = await registry.executeTool("fs.read", { path: "test.txt" });
    assert.ok((verifyRead.content as { content: string }).content.includes("mango"));
  } finally {
    await fs.rm(tmpDir, { recursive: true, force: true });
  }
});

test("fs.list, fs.glob, fs.search, fs.move and fs.delete", async () => {
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "forge-fs-ops-"));
  try {
    const sandbox = new ProjectSandbox(tmpDir);
    const registry = new ToolRegistry();
    registry.registerTools(createFsTools(sandbox));

    await registry.executeTool("fs.write", { path: "a.ts", content: "const hello = 'world';" });
    await registry.executeTool("fs.write", { path: "sub/b.ts", content: "const test = 42;" });

    const listRes = await registry.executeTool("fs.list", { path: "." });
    assert.equal(listRes.is_error, false);
    const listItems = listRes.content as Array<{ name: string }>;
    assert.ok(listItems.some((i) => i.name === "a.ts"));

    const globRes = await registry.executeTool("fs.glob", { pattern: "**/*.ts" });
    assert.equal(globRes.is_error, false);
    const globFiles = globRes.content as string[];
    assert.ok(globFiles.includes("a.ts"));
    assert.ok(globFiles.includes("sub/b.ts"));

    const searchRes = await registry.executeTool("fs.search", { query: "const hello" });
    assert.equal(searchRes.is_error, false);
    const matches = searchRes.content as Array<{ file: string; line_number: number }>;
    assert.equal(matches.length, 1);
    assert.equal(matches[0].file, "a.ts");
    assert.equal(matches[0].line_number, 1);

    const moveRes = await registry.executeTool("fs.move", { source: "a.ts", target: "moved.ts" });
    assert.equal(moveRes.is_error, false);

    const delRes = await registry.executeTool("fs.delete", { path: "moved.ts" });
    assert.equal(delRes.is_error, false);

    const checkRead = await registry.executeTool("fs.read", { path: "moved.ts" });
    assert.equal(checkRead.is_error, true);
    assert.equal((checkRead.content as { code: string }).code, "FILE_NOT_FOUND");
  } finally {
    await fs.rm(tmpDir, { recursive: true, force: true });
  }
});
