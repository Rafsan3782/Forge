import { spawn } from "node:child_process";
import path from "node:path";
import fs from "node:fs/promises";
import type { ToolDefinition } from "./fs_tools.ts";
import { ToolExecutionError } from "./errors.ts";
import { truncateOutput } from "./types.ts";

export interface CommandRunnerOptions {
  projectRoot: string;
  forgeCliPath?: string;
  forgeRuntimePath?: string;
}

export function createCommandTools(options: CommandRunnerOptions): ToolDefinition[] {
  const projectRoot = path.resolve(options.projectRoot);
  const cliBinary = options.forgeCliPath ?? "forge";
  const runtimeBinary = options.forgeRuntimePath ?? "forge_runtime";

  return [
    {
      descriptor: {
        name: "forge.check",
        description: "Run 'forge check --json' on the game project to validate scripts, scenes, and asset references.",
        category: "exec",
        risk_level: "low",
        input_schema: {
          type: "object",
          properties: {
            project_path: {
              type: "string",
              description: "Optional project path relative to workspace or project root (default: current project)",
            },
          },
        },
      },
      metadata: {
        category: "exec",
        risk: "low",
        timeout: 30000,
        idempotent: true,
      },
      execute: async (args, signal) => {
        const targetProj = args.project_path ? path.resolve(projectRoot, String(args.project_path)) : projectRoot;
        const res = await execSubprocess(
          cliBinary,
          ["check", "--project", targetProj, "--json"],
          { cwd: projectRoot, timeoutMs: 30000, signal }
        );

        try {
          const parsed = JSON.parse(res.stdout);
          return parsed;
        } catch {
          return {
            raw_stdout: truncateOutput(res.stdout, 5000),
            raw_stderr: truncateOutput(res.stderr, 5000),
            exit_code: res.exitCode,
          };
        }
      },
    },
    {
      descriptor: {
        name: "forge.run_headless",
        description: "Run game in headless mode for a fixed number of frames, write viewport screenshot, and capture output.",
        category: "exec",
        risk_level: "medium",
        input_schema: {
          type: "object",
          properties: {
            frames: { type: "integer", description: "Number of frames to step before capturing screenshot (default: 5)" },
            screenshot_path: { type: "string", description: "Path where screenshot PNG will be saved" },
            scene: { type: "string", description: "Scene file path to run (relative to project)" },
            scripted_input: { type: "string", description: "Optional path to scripted input JSON" },
          },
        },
      },
      metadata: {
        category: "exec",
        risk: "medium",
        timeout: 60000,
        idempotent: false,
      },
      execute: async (args, signal) => {
        const frames = args.frames !== undefined ? Number(args.frames) : 5;
        const screenshotPath = args.screenshot_path ? String(args.screenshot_path) : "screenshot.png";
        const resolvedScreenshot = path.isAbsolute(screenshotPath)
          ? screenshotPath
          : path.resolve(projectRoot, screenshotPath);

        const cmdArgs = [
          "--headless",
          "--frames",
          String(frames),
          "--screenshot",
          resolvedScreenshot,
        ];

        if (args.scene) {
          cmdArgs.push("--scene", String(args.scene));
        }
        if (args.scripted_input) {
          cmdArgs.push("--input", String(args.scripted_input));
        }

        const res = await execSubprocess(runtimeBinary, cmdArgs, {
          cwd: projectRoot,
          timeoutMs: 60000,
          signal,
        });

        let screenshotExists = false;
        try {
          await fs.access(resolvedScreenshot);
          screenshotExists = true;
        } catch {
          screenshotExists = false;
        }

        return {
          success: res.exitCode === 0,
          exit_code: res.exitCode,
          frames_simulated: frames,
          screenshot_saved: screenshotExists,
          screenshot_path: screenshotPath,
          logs: truncateOutput(res.stdout, 5000),
          errors: truncateOutput(res.stderr, 5000),
        };
      },
    },
    {
      descriptor: {
        name: "forge.build",
        description: "Build the game project assets and standalone distribution if required.",
        category: "exec",
        risk_level: "medium",
        input_schema: {
          type: "object",
          properties: {
            mode: { type: "string", enum: ["debug", "release"], description: "Build mode (default: debug)" },
          },
        },
      },
      metadata: {
        category: "exec",
        risk: "medium",
        timeout: 120000,
        idempotent: true,
      },
      execute: async (args, signal) => {
        const mode = args.mode === "release" ? "release" : "debug";
        const res = await execSubprocess(
          cliBinary,
          ["build", "--project", projectRoot, "--mode", mode],
          { cwd: projectRoot, timeoutMs: 120000, signal }
        );

        return {
          success: res.exitCode === 0,
          exit_code: res.exitCode,
          output: truncateOutput(res.stdout, 5000),
          errors: truncateOutput(res.stderr, 5000),
        };
      },
    },
  ];
}

interface ExecResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}

function execSubprocess(
  command: string,
  args: string[],
  options: { cwd: string; timeoutMs?: number; signal?: AbortSignal }
): Promise<ExecResult> {
  return new Promise((resolve, reject) => {
    let stdoutData = "";
    let stderrData = "";
    let isSettled = false;

    const child = spawn(command, args, {
      cwd: options.cwd,
      shell: false,
    });

    const timeoutTimer = options.timeoutMs
      ? setTimeout(() => {
          if (!isSettled) {
            isSettled = true;
            child.kill("SIGKILL");
            reject(
              new ToolExecutionError(
                "COMMAND_TIMEOUT",
                `Command '${command}' timed out after ${options.timeoutMs}ms`,
                "Increase tool timeout or inspect the process logs."
              )
            );
          }
        }, options.timeoutMs)
      : null;

    if (options.signal) {
      options.signal.addEventListener("abort", () => {
        if (!isSettled) {
          isSettled = true;
          if (timeoutTimer) clearTimeout(timeoutTimer);
          child.kill("SIGINT");
          reject(
            new ToolExecutionError(
              "OPERATION_ABORTED",
              `Command execution was aborted by caller signal`,
              "Retry when ready."
            )
          );
        }
      });
    }

    child.stdout.on("data", (chunk: Buffer) => {
      stdoutData += chunk.toString("utf8");
    });

    child.stderr.on("data", (chunk: Buffer) => {
      stderrData += chunk.toString("utf8");
    });

    child.on("error", (err) => {
      if (!isSettled) {
        isSettled = true;
        if (timeoutTimer) clearTimeout(timeoutTimer);
        reject(
          new ToolExecutionError(
            "SPAWN_ERROR",
            `Failed to start command '${command}': ${err.message}`,
            "Ensure the binary is installed or on PATH."
          )
        );
      }
    });

    child.on("close", (code) => {
      if (!isSettled) {
        isSettled = true;
        if (timeoutTimer) clearTimeout(timeoutTimer);
        resolve({
          stdout: stdoutData,
          stderr: stderrData,
          exitCode: code ?? 0,
        });
      }
    });
  });
}
