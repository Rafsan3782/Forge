import { BridgeClient, type BridgeClientOptions } from "../client.ts";
import { convertJsonSchemaToGemini } from "./schema_converter.ts";
import type { ToolDefinition } from "./fs_tools.ts";
import { ToolExecutionError } from "./errors.ts";
import { formatScreenshotAsImagePart } from "./vision.ts";

export interface McpBridgeTool {
  name: string;
  description: string;
  input_schema?: Record<string, unknown>;
  parameters?: Record<string, unknown>;
  category?: "read" | "write" | "exec" | "network";
  risk_level?: "safe" | "low" | "medium" | "high" | "critical";
  timeout?: number;
  idempotent?: boolean;
}

export interface EngineToolsOptions extends BridgeClientOptions {
  autoReconnect?: boolean;
  reconnectInterval?: number;
  maxScreenshotDimension?: number;
}

export class EngineMcpClient {
  private client: BridgeClient;
  private knownTools: Map<string, ToolDefinition> = new Map();
  public readonly maxScreenshotDimension: number;

  constructor(options: EngineToolsOptions) {
    this.maxScreenshotDimension = options.maxScreenshotDimension ?? 1024;
    this.client = new BridgeClient({
      host: options.host,
      port: options.port,
      token: options.token,
      autoReconnect: options.autoReconnect ?? true,
      reconnectInterval: options.reconnectInterval ?? 2000,
    });
  }

  public async connect(): Promise<void> {
    await this.client.connect();
    await this.refreshTools();
  }

  public disconnect(): void {
    this.client.disconnect();
  }

  public isConnected(): boolean {
    return this.client.isConnected();
  }

  public getClient(): BridgeClient {
    return this.client;
  }

  public async refreshTools(): Promise<ToolDefinition[]> {
    if (!this.client.isConnected()) {
      return Array.from(this.knownTools.values());
    }

    try {
      const response = await this.client.call<{ tools: McpBridgeTool[] }>(
        "tools/list",
        {}
      );

      this.knownTools.clear();
      for (const rawTool of response.tools ?? []) {
        const toolDef = this.createToolDefinitionFromMcp(rawTool);
        this.knownTools.set(toolDef.descriptor.name, toolDef);
      }
    } catch {
      this.registerDefaultEngineTools();
    }

    return Array.from(this.knownTools.values());
  }

  public getTools(): ToolDefinition[] {
    return Array.from(this.knownTools.values());
  }

  public async executeTool(
    name: string,
    args: Record<string, unknown>,
    signal?: AbortSignal
  ): Promise<unknown> {
    const tool = this.knownTools.get(name);
    if (!tool) {
      throw new ToolExecutionError(
        "TOOL_NOT_FOUND",
        `Engine tool '${name}' is not registered or supported by engine bridge`,
        "Check available tools via listTools."
      );
    }

    if (!this.client.isConnected()) {
      throw new ToolExecutionError(
        "BRIDGE_DISCONNECTED",
        `Cannot execute '${name}': engine bridge is disconnected`,
        "Wait for engine bridge reconnection or check that the game engine is running."
      );
    }

    return tool.execute(args, signal);
  }

  private createToolDefinitionFromMcp(tool: McpBridgeTool): ToolDefinition {
    const rawSchema = tool.input_schema ?? tool.parameters ?? { type: "object" };
    const convertedSchema = convertJsonSchemaToGemini(rawSchema);

    const name = tool.name;
    const isScreenshot = name === "game.screenshot";

    return {
      descriptor: {
        name,
        description: tool.description || `Engine operation: ${name}`,
        input_schema: convertedSchema,
        category: tool.category ?? (name.startsWith("scene.save") || name.startsWith("scene.create") ? "write" : "read"),
        risk_level: tool.risk_level ?? "safe",
      },
      metadata: {
        category: tool.category ?? "read",
        risk: tool.risk_level ?? "safe",
        timeout: tool.timeout ?? 10000,
        idempotent: tool.idempotent ?? false,
      },
      execute: async (args) => {
        const res = await this.client.call<Record<string, unknown>>(name, args);
        if (isScreenshot && res && typeof res === "object") {
          const screenshotRes = res as { data_base64?: string; width?: number; height?: number };
          if (screenshotRes.data_base64) {
            return {
              ...screenshotRes,
              image_part: formatScreenshotAsImagePart(screenshotRes.data_base64),
            };
          }
        }
        return res;
      },
    };
  }

  private registerDefaultEngineTools(): void {
    const defaultTools: McpBridgeTool[] = [
      {
        name: "engine.ping",
        description: "Heartbeat and liveness check for the Forge engine",
        input_schema: {
          type: "object",
          properties: { timestamp: { type: "number" } },
        },
        category: "read",
        risk_level: "safe",
        idempotent: true,
      },
      {
        name: "scene.get_tree",
        description: "Retrieve hierarchy tree of entities in current scene",
        input_schema: {
          type: "object",
          properties: { root_entity_id: { type: ["string", "null"] } },
        },
        category: "read",
        risk_level: "safe",
        idempotent: true,
      },
      {
        name: "scene.create_entity",
        description: "Create a new entity in the scene",
        input_schema: {
          type: "object",
          required: ["name"],
          properties: {
            name: { type: "string" },
            parent_id: { type: ["string", "null"] },
            components: { type: "object" },
          },
        },
        category: "write",
        risk_level: "low",
        idempotent: false,
      },
      {
        name: "scene.set_component",
        description: "Attach or update a component on an entity",
        input_schema: {
          type: "object",
          required: ["entity_id", "component_name", "data"],
          properties: {
            entity_id: { type: "string" },
            component_name: { type: "string" },
            data: { type: "object" },
          },
        },
        category: "write",
        risk_level: "low",
        idempotent: true,
      },
      {
        name: "scene.delete_entity",
        description: "Delete an entity and its descendants from the scene",
        input_schema: {
          type: "object",
          required: ["entity_id"],
          properties: { entity_id: { type: "string" } },
        },
        category: "write",
        risk_level: "medium",
        idempotent: false,
      },
      {
        name: "game.screenshot",
        description: "Capture a viewport screenshot from the running game instance",
        input_schema: {
          type: "object",
          properties: {
            format: { type: "string", enum: ["png", "jpeg"] },
            width: { type: ["integer", "null"] },
            height: { type: ["integer", "null"] },
            file_path: { type: ["string", "null"] },
          },
        },
        category: "read",
        risk_level: "safe",
        idempotent: true,
      },
    ];

    this.knownTools.clear();
    for (const t of defaultTools) {
      const def = this.createToolDefinitionFromMcp(t);
      this.knownTools.set(def.descriptor.name, def);
    }
  }
}
