import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { ToolCategory, ToolRiskLevel } from "@forge/protocol";

export type PermissionDecision = "approve_once" | "always_allow" | "deny";

export interface PermissionRule {
  id: string;
  category?: ToolCategory;
  tool_name?: string;
  path_glob?: string;
  command?: string;
  created_at: number;
}

export interface PermissionsStore {
  version: number;
  rules: PermissionRule[];
}

export class PermissionsManager {
  private projectRoot: string;
  private permissionsFilePath: string;
  private cachedRules: PermissionRule[] | null = null;

  constructor(projectRoot: string) {
    this.projectRoot = path.resolve(projectRoot);
    this.permissionsFilePath = path.join(this.projectRoot, ".forge", "permissions.json");
  }

  public async getRules(): Promise<PermissionRule[]> {
    if (this.cachedRules !== null) {
      return this.cachedRules;
    }
    try {
      const content = await fs.readFile(this.permissionsFilePath, "utf8");
      const parsed: PermissionsStore = JSON.parse(content);
      this.cachedRules = Array.isArray(parsed.rules) ? parsed.rules : [];
      return this.cachedRules;
    } catch {
      this.cachedRules = [];
      return [];
    }
  }

  public async addRule(rule: Omit<PermissionRule, "id" | "created_at">): Promise<PermissionRule> {
    const rules = await this.getRules();
    const newRule: PermissionRule = {
      ...rule,
      id: `rule_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
      created_at: Date.now(),
    };
    rules.push(newRule);
    this.cachedRules = rules;
    await this.persist();
    return newRule;
  }

  public async removeRule(ruleId: string): Promise<boolean> {
    const rules = await this.getRules();
    const initialLen = rules.length;
    this.cachedRules = rules.filter((r) => r.id !== ruleId);
    if (this.cachedRules.length !== initialLen) {
      await this.persist();
      return true;
    }
    return false;
  }

  public async clearRules(): Promise<void> {
    this.cachedRules = [];
    await this.persist();
  }

  private async persist(): Promise<void> {
    const dir = path.dirname(this.permissionsFilePath);
    try {
      await fs.mkdir(dir, { recursive: true });
      const store: PermissionsStore = {
        version: 1,
        rules: this.cachedRules || [],
      };
      await fs.writeFile(this.permissionsFilePath, JSON.stringify(store, null, 2), "utf8");
    } catch {
      // Ignore write errors in read-only environments
    }
  }

  public matchesRule(
    toolName: string,
    category: ToolCategory,
    args: Record<string, unknown>
  ): boolean {
    if (!this.cachedRules || this.cachedRules.length === 0) {
      return false;
    }

    const targetPath = (args.path || args.file_path || args.source_path || args.project_path) as string | undefined;
    const command = (args.command || args.cmd) as string | undefined;

    for (const rule of this.cachedRules) {
      if (rule.tool_name && rule.tool_name !== toolName) {
        continue;
      }
      if (rule.category && rule.category !== category) {
        continue;
      }
      if (rule.command && command) {
        if (!this.matchCommand(command, rule.command)) {
          continue;
        }
      }
      if (rule.path_glob && targetPath) {
        if (!this.matchGlob(targetPath, rule.path_glob)) {
          continue;
        }
      }
      return true;
    }

    return false;
  }

  private matchCommand(actual: string, pattern: string): boolean {
    const trimmedActual = actual.trim();
    const trimmedPattern = pattern.trim();
    if (trimmedActual === trimmedPattern) return true;
    if (trimmedActual.startsWith(trimmedPattern + " ")) return true;
    return false;
  }

  public matchGlob(targetPath: string, pattern: string): boolean {
    const normalized = targetPath.replace(/\\/g, "/").replace(/^\.\//, "");
    const normalizedPattern = pattern.replace(/\\/g, "/").replace(/^\.\//, "");

    if (normalizedPattern === "*" || normalizedPattern === "**/*") return true;
    if (normalized === normalizedPattern) return true;

    const regexPattern = normalizedPattern
      .replace(/[.+^${}()|[\]\\]/g, "\\$&")
      .replace(/\*\*/g, ".*")
      .replace(/\*/g, "[^/]*");

    const regex = new RegExp(`^${regexPattern}$`);
    return regex.test(normalized);
  }
}
