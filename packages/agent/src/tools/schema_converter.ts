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
 * Handles recursive $ref (such as EntityNode children) without infinite recursion.
 */

export function convertJsonSchemaToGemini(
  rootSchema: Record<string, unknown>,
  targetSchema?: Record<string, unknown>
): Record<string, unknown> {
  const schemaToConvert = targetSchema ?? rootSchema;
  const visitedRefs = new Set<string>();

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

  function transformNode(input: unknown, activePath = new Set<string>()): unknown {
    if (!input || typeof input !== "object" || Array.isArray(input)) {
      return input;
    }

    let node = input as Record<string, unknown>;

    // Handle $ref
    if (typeof node.$ref === "string") {
      const refStr = node.$ref;
      if (activePath.has(refStr)) {
        // Recursive cycle detected (e.g. EntityNode.children -> EntityNode)
        return {
          type: "object",
          description: typeof node.description === "string" ? node.description : `Recursive reference to ${refStr}`,
        };
      }

      const deref = dereference(refStr);
      if (deref) {
        const nextActive = new Set(activePath);
        nextActive.add(refStr);
        const merged = { ...deref };
        if (typeof node.description === "string") {
          merged.description = node.description;
        }
        return transformNode(merged, nextActive);
      }
    }

    const result: Record<string, unknown> = {};

    // 1. Description
    if (typeof node.description === "string") {
      result.description = node.description;
    }

    // 2. Type & Nullable handling
    if (node.type !== undefined) {
      if (Array.isArray(node.type)) {
        const types = node.type as string[];
        const hasNull = types.includes("null");
        const nonNullTypes = types.filter((t) => t !== "null");
        if (nonNullTypes.length > 0) {
          result.type = nonNullTypes[0];
        }
        if (hasNull) {
          result.nullable = true;
        }
      } else if (typeof node.type === "string") {
        if (node.type === "null") {
          result.type = "string";
          result.nullable = true;
        } else {
          result.type = node.type;
        }
      }
    }

    // 3. Nullable explicit
    if (node.nullable === true) {
      result.nullable = true;
    }

    // 4. Format
    if (typeof node.format === "string") {
      result.format = node.format;
    }

    // 5. Enum
    if (Array.isArray(node.enum)) {
      result.enum = node.enum;
    }

    // 6. Properties
    if (node.properties && typeof node.properties === "object" && !Array.isArray(node.properties)) {
      const propsObj: Record<string, unknown> = {};
      for (const [key, propVal] of Object.entries(node.properties as Record<string, unknown>)) {
        propsObj[key] = transformNode(propVal, activePath);
      }
      result.properties = propsObj;
      if (!result.type) {
        result.type = "object";
      }
    }

    // 7. Items
    if (node.items) {
      if (Array.isArray(node.items)) {
        result.items = transformNode(node.items[0], activePath);
      } else {
        result.items = transformNode(node.items, activePath);
      }
      if (!result.type) {
        result.type = "array";
      }
    }

    // 8. Required
    if (Array.isArray(node.required)) {
      result.required = node.required;
    }

    return result;
  }

  return (transformNode(schemaToConvert) as Record<string, unknown>) ?? { type: "object" };
}
