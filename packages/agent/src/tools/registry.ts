import type { ToolDescriptor } from "@forge/protocol";
import type { ToolProvider, ToolResult } from "../contracts/interfaces.ts";
import type { ToolDefinition } from "./fs_tools.ts";
import { ToolExecutionError, formatToolError } from "./errors.ts";
import type { ToolMetadata } from "./types.ts";

export class ToolRegistry implements ToolProvider {
  private tools: Map<string, ToolDefinition> = new Map();

  public registerTool(tool: ToolDefinition): void {
    if (this.tools.has(tool.descriptor.name)) {
      throw new ToolExecutionError(
        "TOOL_COLLISION",
        `Tool with name '${tool.descriptor.name}' is already registered`,
        "Rename the colliding tool or avoid duplicate registration."
      );
    }
    this.tools.set(tool.descriptor.name, tool);
  }

  public registerTools(tools: ToolDefinition[]): void {
    for (const tool of tools) {
      this.registerTool(tool);
    }
  }

  public registerProviderTools(providerTools: ToolDefinition[]): void {
    this.registerTools(providerTools);
  }

  public getTool(name: string): ToolDefinition | undefined {
    return this.tools.get(name);
  }

  public getToolMetadata(name: string): ToolMetadata | undefined {
    return this.tools.get(name)?.metadata;
  }

  // --- ToolProvider Interface Implementation ---

  public async listTools(): Promise<ToolDescriptor[]> {
    return Array.from(this.tools.values()).map((t) => t.descriptor);
  }

  public async executeTool(
    name: string,
    args: Record<string, unknown>,
    signal?: AbortSignal
  ): Promise<ToolResult> {
    const tool = this.tools.get(name);
    if (!tool) {
      const structuredErr = new ToolExecutionError(
        "TOOL_NOT_FOUND",
        `Unknown tool: '${name}'`,
        "Check available tools with listTools()."
      ).toStructured();

      return {
        content: structuredErr,
        is_error: true,
      };
    }

    try {
      const content = await tool.execute(args, signal);
      return {
        content,
        is_error: false,
      };
    } catch (err: unknown) {
      return {
        content: formatToolError(err),
        is_error: true,
      };
    }
  }
}
