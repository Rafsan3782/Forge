import * as path from "node:path";
import type { ToolDescriptor, AgentSettings, PermissionRequest } from "@forge/protocol";
import type { SafetyPolicy, SafetyCheckResult } from "../contracts/interfaces.ts";
import { PermissionsManager, type PermissionDecision } from "./permissions.ts";

export interface StandardSafetyPolicyOptions {
  projectRoot: string;
  defaultTimeoutMs?: number;
}

export interface PendingPermissionRequest {
  request: PermissionRequest;
  resolve: (decision: PermissionDecision) => void;
  reject: (err: Error) => void;
  timeoutTimer: NodeJS.Timeout;
}

export class StandardSafetyPolicy implements SafetyPolicy {
  private projectRoot: string;
  private permissionsManager: PermissionsManager;
  private defaultTimeoutMs: number;
  private pendingRequests: Map<string, PendingPermissionRequest> = new Map();
  private isPaused = false;
  private isStopped = false;
  private onPermissionRequested?: (request: PermissionRequest) => void;

  constructor(options: StandardSafetyPolicyOptions) {
    this.projectRoot = path.resolve(options.projectRoot);
    this.permissionsManager = new PermissionsManager(this.projectRoot);
    this.defaultTimeoutMs = options.defaultTimeoutMs ?? 30000;
  }

  public getPermissionsManager(): PermissionsManager {
    return this.permissionsManager;
  }

  public setOnPermissionRequested(callback: (request: PermissionRequest) => void): void {
    this.onPermissionRequested = callback;
  }

  public isHardDenied(
    tool: ToolDescriptor,
    args: Record<string, unknown>
  ): { denied: boolean; reason?: string } {
    const rawPaths = [
      args.path,
      args.file_path,
      args.source_path,
      args.target_path,
      args.project_path,
      args.root_path,
    ].filter((p): p is string => typeof p === "string");

    for (const rawPath of rawPaths) {
      const resolved = path.resolve(this.projectRoot, rawPath);
      const relative = path.relative(this.projectRoot, resolved);

      // Check 1: paths outside the project root
      if (relative.startsWith("..") || path.isAbsolute(relative)) {
        return {
          denied: true,
          reason: `Access to paths outside the project root is strictly prohibited: ${rawPath}`,
        };
      }

      const normalizedRel = relative.replace(/\\/g, "/");

      // Check 2: .git internals
      if (normalizedRel === ".git" || normalizedRel.startsWith(".git/")) {
        return {
          denied: true,
          reason: `Access to .git internal directories is strictly prohibited: ${rawPath}`,
        };
      }

      // Check 3: .forge/secrets
      if (normalizedRel === ".forge/secrets" || normalizedRel.startsWith(".forge/secrets/")) {
        return {
          denied: true,
          reason: `Access to .forge/secrets is strictly prohibited: ${rawPath}`,
        };
      }

      // Check 4: deleting the project root
      if (
        (tool.name === "fs.delete" ||
          tool.name === "file.delete" ||
          tool.name === "delete_file" ||
          tool.name === "delete_directory") &&
        (normalizedRel === "" || normalizedRel === ".")
      ) {
        return {
          denied: true,
          reason: "Deleting the project root is strictly prohibited",
        };
      }
    }

    // Check command execution for root deletion / git internals
    if (tool.category === "exec" && typeof args.command === "string") {
      const cmd = args.command.toLowerCase();
      if (
        cmd.includes("rm -rf /") ||
        cmd.includes("rmdir /s /q .") ||
        cmd.includes("rm -rf .git") ||
        cmd.includes("del /f /s /q .git")
      ) {
        return {
          denied: true,
          reason: `Destructive command violates safety policy: ${args.command}`,
        };
      }
    }

    return { denied: false };
  }

  public async evaluate(
    tool: ToolDescriptor,
    args: Record<string, unknown>,
    settings: AgentSettings
  ): Promise<SafetyCheckResult> {
    // 1. Hard deny list evaluation (can NEVER be overridden)
    const hardDeny = this.isHardDenied(tool, args);
    if (hardDeny.denied) {
      return {
        allowed: false,
        requires_permission: false,
        risk_level: "critical",
        reason: hardDeny.reason,
      };
    }

    // 2. Check persistent Always-Allow rules
    await this.permissionsManager.getRules();
    if (this.permissionsManager.matchesRule(tool.name, tool.category, args)) {
      return {
        allowed: true,
        requires_permission: false,
        risk_level: tool.risk_level,
        reason: "Matched always-allow permission rule",
      };
    }

    // 3. Evaluate by permission mode
    // Settings modes: "autonomous" (Full auto), "ask_risky" (Auto-edit), "ask_all" (Ask)
    const mode = settings.permission_mode;

    if (mode === "autonomous") {
      // Full auto: everything is allowed automatically except hard deny list
      return {
        allowed: true,
        requires_permission: false,
        risk_level: tool.risk_level,
      };
    }

    if (mode === "ask_risky") {
      // Auto-edit: file and scene edits are automatic (safe or read or write if editing scripts/scenes)
      // Exec and network ask.
      if (tool.category === "read") {
        return {
          allowed: true,
          requires_permission: false,
          risk_level: tool.risk_level,
        };
      }
      if (tool.category === "write") {
        const isHighRisk = tool.risk_level === "high" || tool.risk_level === "critical";
        return {
          allowed: true,
          requires_permission: isHighRisk,
          risk_level: tool.risk_level,
          reason: isHighRisk ? `High risk write operation: ${tool.name}` : undefined,
        };
      }
      return {
        allowed: true,
        requires_permission: true,
        risk_level: tool.risk_level,
        reason: `${tool.category.toUpperCase()} action requires approval in Auto-edit mode`,
      };
    }

    // Default: "ask_all" (Ask: every write and exec needs approval)
    const needsApproval = tool.category === "write" || tool.category === "exec" || tool.category === "network" || tool.risk_level !== "safe";
    return {
      allowed: true,
      requires_permission: needsApproval,
      risk_level: tool.risk_level,
      reason: needsApproval ? `Operation requires user approval in Ask mode: ${tool.name}` : undefined,
    };
  }

  public async requestPermission(
    request: PermissionRequest,
    timeoutMs: number = this.defaultTimeoutMs
  ): Promise<PermissionDecision> {
    if (this.isStopped) {
      return "deny";
    }

    return new Promise<PermissionDecision>((resolve, reject) => {
      let resolved = false;

      const timer = setTimeout(() => {
        if (!resolved) {
          resolved = true;
          this.pendingRequests.delete(request.id);
          resolve("deny");
        }
      }, timeoutMs);

      this.pendingRequests.set(request.id, {
        request,
        resolve: (decision) => {
          if (!resolved) {
            resolved = true;
            clearTimeout(timer);
            this.pendingRequests.delete(request.id);
            resolve(decision);
          }
        },
        reject: (err) => {
          if (!resolved) {
            resolved = true;
            clearTimeout(timer);
            this.pendingRequests.delete(request.id);
            reject(err);
          }
        },
        timeoutTimer: timer,
      });

      if (this.onPermissionRequested) {
        try {
          this.onPermissionRequested(request);
        } catch {
          // ignore callback error
        }
      }
    });
  }

  public resolvePermission(requestId: string, decision: PermissionDecision): boolean {
    const pending = this.pendingRequests.get(requestId);
    if (!pending) {
      return false;
    }
    pending.resolve(decision);
    return true;
  }

  public pause(): void {
    this.isPaused = true;
  }

  public resume(): void {
    this.isPaused = false;
  }

  public stop(): void {
    this.isStopped = true;
    for (const pending of this.pendingRequests.values()) {
      clearTimeout(pending.timeoutTimer);
      pending.resolve("deny");
    }
    this.pendingRequests.clear();
  }

  public getPendingRequests(): PermissionRequest[] {
    return Array.from(this.pendingRequests.values()).map((p) => p.request);
  }
}
