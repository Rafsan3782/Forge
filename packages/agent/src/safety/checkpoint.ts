import * as child_process from "node:child_process";
import * as util from "node:util";
import * as path from "node:path";
import type { Checkpoint } from "@forge/protocol";
import type { CheckpointManager } from "../contracts/interfaces.ts";

const execFileAsync = util.promisify(child_process.execFile);

export interface GitCheckpointManagerOptions {
  projectRoot: string;
  uncommittedStrategy?: "snapshot" | "refuse";
}

export interface CheckpointDiffSummary {
  files_changed: number;
  insertions: number;
  deletions: number;
  file_details: Array<{
    path: string;
    status: "added" | "modified" | "deleted";
    insertions: number;
    deletions: number;
  }>;
}

export class GitCheckpointManager implements CheckpointManager {
  private projectRoot: string;
  private uncommittedStrategy: "snapshot" | "refuse";
  private initialized = false;
  private initialBranch: string = "main";
  private activeBranch?: string;

  constructor(options: GitCheckpointManagerOptions) {
    this.projectRoot = path.resolve(options.projectRoot);
    this.uncommittedStrategy = options.uncommittedStrategy ?? "snapshot";
  }

  private async git(args: string[]): Promise<{ stdout: string; stderr: string }> {
    try {
      const parentDir = path.dirname(this.projectRoot);
      return await execFileAsync("git", args, {
        cwd: this.projectRoot,
        env: {
          ...process.env,
          GIT_CEILING_DIRECTORIES: parentDir,
        },
      });
    } catch (err: unknown) {
      const e = err as { stdout?: string; stderr?: string; message?: string };
      throw new Error(`git ${args.join(" ")} failed: ${e.stderr || e.message || String(err)}`);
    }
  }

  public async ensureInitialized(runId: string): Promise<void> {
    if (this.initialized) return;

    // 1. Check if git repo exists directly in this projectRoot; if not, initialize one
    let isRepo = false;
    try {
      const gitDir = path.join(this.projectRoot, ".git");
      const stat = await fs.stat(gitDir);
      if (stat.isDirectory()) {
        isRepo = true;
      }
    } catch {
      isRepo = false;
    }

    if (!isRepo) {
      await this.git(["init"]);
      await this.git(["config", "user.name", "Forge Agent"]);
      await this.git(["config", "user.email", "agent@forge.local"]);
      await this.git(["commit", "--allow-empty", "-m", "chore: initialize forge project repository"]);
    }

    // Get current branch
    try {
      const { stdout } = await this.git(["branch", "--show-current"]);
      this.initialBranch = stdout.trim() || "main";
    } catch {
      this.initialBranch = "main";
    }

    // 2. Handle uncommitted user changes
    const isDirty = await this.hasUncommittedChanges();
    if (isDirty) {
      if (this.uncommittedStrategy === "refuse") {
        throw new Error("Working tree has uncommitted user changes and uncommittedStrategy is 'refuse'");
      } else {
        await this.git(["add", "-A"]);
        await this.git(["commit", "-m", "chore(snapshot): save uncommitted user changes before agent run", "--no-verify"]);
      }
    }

    // 3. Switch to branch forge/agent/<runId>
    const agentBranch = `forge/agent/${runId}`;
    try {
      await this.git(["checkout", "-b", agentBranch]);
    } catch {
      await this.git(["checkout", agentBranch]);
    }
    this.activeBranch = agentBranch;
    this.initialized = true;
  }

  public async hasUncommittedChanges(): Promise<boolean> {
    try {
      const { stdout } = await this.git(["status", "--porcelain"]);
      return stdout.trim().length > 0;
    } catch {
      return false;
    }
  }

  public async createCheckpoint(
    runId: string,
    description: string,
    affectedFiles?: string[]
  ): Promise<Checkpoint> {
    await this.ensureInitialized(runId);

    if (affectedFiles && affectedFiles.length > 0) {
      for (const file of affectedFiles) {
        try {
          await this.git(["add", file]);
        } catch {
          // ignore
        }
      }
    } else {
      await this.git(["add", "-A"]);
    }

    const commitMsg = `${description}\n\nForge-Run: ${runId}\nForge-Checkpoint: true`;
    await this.git(["commit", "--allow-empty", "-m", commitMsg]);

    const { stdout: hashOut } = await this.git(["rev-parse", "HEAD"]);
    const gitRef = hashOut.trim();
    const checkpointId = `chk_${gitRef.slice(0, 10)}`;

    const checkpoint: Checkpoint = {
      id: checkpointId,
      run_id: runId,
      timestamp: Date.now(),
      git_ref: gitRef,
      description,
      affected_files: affectedFiles || (await this.getCommitAffectedFiles(gitRef)),
    };

    return checkpoint;
  }

  private async getCommitAffectedFiles(commitRef: string): Promise<string[]> {
    try {
      const { stdout } = await this.git(["diff-tree", "--no-commit-id", "--name-only", "-r", commitRef]);
      return stdout.split("\n").map((f) => f.trim()).filter((f) => f.length > 0);
    } catch {
      return [];
    }
  }

  public async listCheckpoints(runId: string): Promise<Checkpoint[]> {
    await this.ensureInitialized(runId);

    try {
      const { stdout } = await this.git([
        "log",
        "--grep=Forge-Run:",
        "--format=%H%x00%B%x00%ct%x1f",
      ]);

      const entries = stdout.split("\x1f").map((s) => s.trim()).filter((s) => s.length > 0);
      const checkpoints: Checkpoint[] = [];

      for (const entry of entries) {
        const [commitHash, fullMessage, timestampStr] = entry.split("\x00");
        if (!commitHash || !fullMessage) continue;

        if (fullMessage.includes(`Forge-Run: ${runId}`)) {
          const lines = fullMessage.split("\n");
          const description = lines[0] || "Checkpoint";
          checkpoints.push({
            id: `chk_${commitHash.slice(0, 10)}`,
            run_id: runId,
            timestamp: Number(timestampStr) * 1000 || Date.now(),
            git_ref: commitHash,
            description,
            affected_files: await this.getCommitAffectedFiles(commitHash),
          });
        }
      }

      return checkpoints;
    } catch {
      return [];
    }
  }

  public async getCheckpointDiff(checkpointRef: string): Promise<CheckpointDiffSummary> {
    try {
      const { stdout: numstatOut } = await this.git(["diff", "--numstat", `${checkpointRef}~1`, checkpointRef]);

      let totalInsertions = 0;
      let totalDeletions = 0;
      const fileDetails: CheckpointDiffSummary["file_details"] = [];

      for (const line of numstatOut.split("\n")) {
        const parts = line.trim().split("\t");
        if (parts.length >= 3) {
          const ins = parts[0] === "-" ? 0 : parseInt(parts[0], 10) || 0;
          const del = parts[1] === "-" ? 0 : parseInt(parts[1], 10) || 0;
          const filePath = parts[2];
          totalInsertions += ins;
          totalDeletions += del;
          fileDetails.push({
            path: filePath,
            status: "modified",
            insertions: ins,
            deletions: del,
          });
        }
      }

      return {
        files_changed: fileDetails.length,
        insertions: totalInsertions,
        deletions: totalDeletions,
        file_details: fileDetails,
      };
    } catch {
      return {
        files_changed: 0,
        insertions: 0,
        deletions: 0,
        file_details: [],
      };
    }
  }

  public async revertToCheckpoint(checkpointId: string): Promise<boolean> {
    try {
      let commitToRestore = checkpointId;
      if (checkpointId.startsWith("chk_")) {
        const shortHash = checkpointId.slice(4);
        try {
          const { stdout: fullHash } = await this.git(["rev-parse", shortHash]);
          commitToRestore = fullHash.trim();
        } catch {
          commitToRestore = shortHash;
        }
      }

      // 1. Detect and snapshot any current uncommitted changes (such as user edits made after the checkpoint)
      const userEditedFiles = new Set<string>();
      try {
        const { stdout: statusOut } = await this.git(["status", "--porcelain"]);
        for (const line of statusOut.split("\n")) {
          const trimmed = line.trim();
          if (trimmed.length > 2) {
            const filePath = trimmed.slice(2).trim();
            userEditedFiles.add(filePath);
          }
        }
      } catch {
        // ignore
      }

      const isDirty = await this.hasUncommittedChanges();
      if (isDirty) {
        await this.git(["add", "-A"]);
        await this.git(["commit", "-m", "chore: snapshot user changes before revert"]);
      }

      // 2. Identify files that were added between commitToRestore and HEAD
      const { stdout: diffFilesOut } = await this.git(["diff", "--name-only", "--diff-filter=A", commitToRestore, "HEAD"]);
      const addedFiles = diffFilesOut.split("\n").map((s) => s.trim()).filter((s) => s.length > 0);

      // 3. Checkout all files from target commit to working tree
      await this.git(["checkout", commitToRestore, "--", "."]);

      // 4. Remove files that were added by agent after commitToRestore, preserving user files
      for (const file of addedFiles) {
        if (userEditedFiles.has(file)) {
          continue; // preserve user files created after checkpoint
        }
        try {
          await this.git(["rm", "-f", file]);
        } catch {
          try {
            await fs.rm(path.join(this.projectRoot, file), { force: true, recursive: true });
          } catch {
            // ignore
          }
        }
      }

      await this.git(["add", "-A"]);
      await this.git([
        "commit",
        "--allow-empty",
        "-m",
        `revert: restore project state to checkpoint ${checkpointId}\n\nRevert-To: ${commitToRestore}`,
      ]);

      return true;
    } catch {
      return false;
    }
  }

  public async squashToMain(targetBranch: string = this.initialBranch): Promise<string> {
    if (!this.activeBranch) {
      throw new Error("No active agent branch to squash");
    }

    await this.git(["checkout", targetBranch]);
    await this.git(["merge", "--squash", this.activeBranch]);
    const squashMsg = `feat(agent): squash merge ${this.activeBranch}`;
    await this.git(["commit", "-m", squashMsg]);

    const { stdout } = await this.git(["rev-parse", "HEAD"]);
    return stdout.trim();
  }
}
