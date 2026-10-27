/**
 * Converts a JSON Schema (such as Draft 2020-12 used in @forge/protocol)
 * to the subset accepted by Gemini for Function Declarations.
 *
 * Supported attributes by Gemini OpenAPI subset:
 * - type
 * - description
 * - properties
 * - items
 * - required
 * - enum
 * - nullable
 * - format
 *
 * Dropped keywords:
 * - $schema, $id, title, default, minItems, maxItems, minimum, maximum,
 *   additionalProperties, definitions, $defs, allOf, oneOf, not, etc.
 *
 * If a type is an array containing "null", e.g. ["string", "null"],
 * it is converted to type: "string" and nullable: true.
 *
 * $ref pointers within the same document (e.g. #/definitions/...) are dereferenced/inlined.
 */

export function convertJsonSchemaToGemini(
  rootSchema: Record<string, unknown>,
  targetSchema?: Record<string, unknown>
): Record<string, unknown> {
  const schemaToConvert = targetSchema ?? rootSchema;

  function dereference(refStr: string): Record<string, unknown> | null {
    if (!refStr.startsWith("#/")) {
      return null;
    }
    const parts = refStr.slice(2).split("/");
    let curr: unknown = rootSchema;
    for (const part of parts) {
      if (curr && typeof curr === "object" && part in curr) {
        curr = (curr as Record<string, unknown>)[part];
      } else {
        return null;
      }
    }
    return curr && typeof curr === "object" ? (curr as Record<string, unknown>) : null;
  }

  function resolveSchema(node: Record<string, unknown>): Record<string, unknown> {
    if (typeof node.$ref === "string") {
      const deref = dereference(node.$ref);
      if (deref) {
        const merged = { ...deref };
        if (typeof node.description === "string") {
          merged.description = node.description;
        }
        return resolveSchema(merged);
      }
    }
    return node;
  }

  function transformNode(input: unknown): unknown {
    if (!input || typeof input !== "object" || Array.isArray(input)) {
      return input;
    }

    const resolved = resolveSchema(input as Record<string, unknown>);
    const result: Record<string, unknown> = {};

    // 1. Description
    if (typeof resolved.description === "string") {
      result.description = resolved.description;
    }

    // 2. Type & Nullable handling
    if (resolved.type !== undefined) {
      if (Array.isArray(resolved.type)) {
        const types = resolved.type as string[];
        const hasNull = types.includes("null");
        const nonNullTypes = types.filter((t) => t !== "null");
        if (nonNullTypes.length > 0) {
          result.type = nonNullTypes[0];
        }
        if (hasNull) {
          result.nullable = true;
        }
      } else if (typeof resolved.type === "string") {
        if (resolved.type === "null") {
          result.type = "string";
          result.nullable = true;
        } else {
          result.type = resolved.type;
        }
      }
    }

    // 3. Nullable explicit
    if (resolved.nullable === true) {
      result.nullable = true;
    }

    // 4. Format
    if (typeof resolved.format === "string") {
      result.format = resolved.format;
    }

    // 5. Enum
    if (Array.isArray(resolved.enum)) {
      result.enum = resolved.enum;
    }

    // 6. Properties (for type object)
    if (resolved.properties && typeof resolved.properties === "object" && !Array.isArray(resolved.properties)) {
      const propsObj: Record<string, unknown> = {};
      for (const [key, propVal] of Object.entries(resolved.properties as Record<string, unknown>)) {
        propsObj[key] = transformNode(propVal);
      }
      result.properties = propsObj;
      if (!result.type) {
        result.type = "object";
      }
    }

    // 7. Items (for type array)
    if (resolved.items) {
      if (Array.isArray(resolved.items)) {
        result.items = transformNode(resolved.items[0]);
      } else {
        result.items = transformNode(resolved.items);
      }
      if (!result.type) {
        result.type = "array";
      }
    }

    // 8. Required array
    if (Array.isArray(resolved.required)) {
      result.required = resolved.required;
    }

    return result;
  }

  return (transformNode(schemaToConvert) as Record<string, unknown>) ?? { type: "object" };
}
