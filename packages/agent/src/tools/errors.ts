export interface ToolErrorData {
  code: string;
  message: string;
  suggested_action?: string;
  details?: unknown;
}

export class ToolExecutionError extends Error {
  public readonly code: string;
  public readonly suggestedAction?: string;
  public readonly details?: unknown;

  constructor(code: string, message: string, suggestedAction?: string, details?: unknown) {
    super(message);
    this.name = "ToolExecutionError";
    this.code = code;
    this.suggestedAction = suggestedAction;
    this.details = details;
  }

  public toStructured(): ToolErrorData {
    return {
      code: this.code,
      message: this.message,
      suggested_action: this.suggestedAction,
      details: this.details,
    };
  }
}

export function formatToolError(error: unknown): ToolErrorData {
  if (error instanceof ToolExecutionError) {
    return error.toStructured();
  }
  if (error instanceof Error) {
    return {
      code: "TOOL_ERROR",
      message: error.message,
      suggested_action: "Check the tool arguments and try again.",
    };
  }
  return {
    code: "UNKNOWN_ERROR",
    message: String(error),
    suggested_action: "Review engine logs and error context.",
  };
}
