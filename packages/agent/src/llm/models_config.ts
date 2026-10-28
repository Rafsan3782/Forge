import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

export type ModelRole = "planner" | "worker" | "fast" | "vision" | "image";

export interface RoleThinkingConfig {
  budget: number;
  include_thoughts: boolean;
}

export interface RoleGenerationConfig {
  temperature: number;
  max_output_tokens: number;
}

export interface RolePricingConfig {
  input_per_million: number;
  output_per_million: number;
  cached_input_per_million?: number;
  cost_per_image?: number;
}

export interface ModelRoleConfig {
  model: string;
  thinking: RoleThinkingConfig;
  generation: RoleGenerationConfig;
  pricing: RolePricingConfig;
}

export interface ModelsConfig {
  roles: Record<ModelRole, ModelRoleConfig>;
}

let cachedConfig: ModelsConfig | null = null;

export function loadModelsConfig(customPath?: string): ModelsConfig {
  if (cachedConfig && !customPath) {
    return cachedConfig;
  }

  let configPath = customPath;
  if (!configPath) {
    const currentDir = path.dirname(fileURLToPath(import.meta.url));
    configPath = path.resolve(currentDir, "../../../config/models.json");
    if (!fs.existsSync(configPath)) {
      configPath = path.resolve(currentDir, "../../config/models.json");
    }
  }

  const raw = fs.readFileSync(configPath, "utf-8");
  const parsed = JSON.parse(raw) as ModelsConfig;

  if (!customPath) {
    cachedConfig = parsed;
  }
  return parsed;
}

export function getRoleConfig(role: ModelRole, customPath?: string): ModelRoleConfig {
  const cfg = loadModelsConfig(customPath);
  const roleCfg = cfg.roles[role];
  if (!roleCfg) {
    throw new Error(`Model role "${role}" not found in models.json`);
  }
  return roleCfg;
}

export function estimateCost(
  roleOrModel: string,
  inputTokens: number,
  outputTokens: number,
  cachedTokens = 0,
  config?: ModelsConfig
): number {
  const cfg = config ?? loadModelsConfig();
  // Find matching role
  let roleCfg = Object.values(cfg.roles).find((r) => r.model === roleOrModel);
  if (!roleCfg && roleOrModel in cfg.roles) {
    roleCfg = cfg.roles[roleOrModel as ModelRole];
  }
  if (!roleCfg) {
    return 0;
  }

  const inputCost = (inputTokens / 1_000_000) * roleCfg.pricing.input_per_million;
  const outputCost = (outputTokens / 1_000_000) * roleCfg.pricing.output_per_million;
  const cachedCost =
    roleCfg.pricing.cached_input_per_million !== undefined
      ? (cachedTokens / 1_000_000) * roleCfg.pricing.cached_input_per_million
      : 0;

  return Number((inputCost + outputCost + cachedCost).toFixed(6));
}
