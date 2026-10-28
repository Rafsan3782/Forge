import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { convertJsonSchemaToGemini } from "./schema_converter.ts";

test("convertJsonSchemaToGemini drops unsupported keywords and keeps descriptions", () => {
  const inputSchema = {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    $id: "https://example.com/test",
    title: "TestSchema",
    description: "A test schema",
    type: "object",
    additionalProperties: false,
    required: ["name", "age"],
    properties: {
      name: {
        type: "string",
        description: "User name",
        default: "Alice",
      },
      age: {
        type: "integer",
        description: "User age",
        minimum: 0,
        maximum: 120,
      },
      tags: {
        type: "array",
        items: {
          type: "string",
        },
        minItems: 1,
      },
    },
  };

  const converted = convertJsonSchemaToGemini(inputSchema);

  assert.equal(converted.description, "A test schema");
  assert.equal(converted.type, "object");
  assert.deepEqual(converted.required, ["name", "age"]);
  assert.equal((converted as Record<string, unknown>).additionalProperties, undefined);
  assert.equal((converted as Record<string, unknown>).$schema, undefined);
  assert.equal((converted as Record<string, unknown>).title, undefined);

  const props = converted.properties as Record<string, Record<string, unknown>>;
  assert.equal(props.name.type, "string");
  assert.equal(props.name.description, "User name");
  assert.equal(props.name.default, undefined);

  assert.equal(props.age.type, "integer");
  assert.equal(props.age.description, "User age");
  assert.equal(props.age.minimum, undefined);

  assert.equal(props.tags.type, "array");
  assert.deepEqual(props.tags.items, { type: "string" });
  assert.equal(props.tags.minItems, undefined);
});

test("convertJsonSchemaToGemini converts nullable array types and resolves $refs", () => {
  const schemaWithRef = {
    definitions: {
      EntityId: {
        type: ["string", "null"],
        description: "Nullable entity ID",
      },
    },
    type: "object",
    properties: {
      entity: {
        $ref: "#/definitions/EntityId",
      },
    },
  };

  const converted = convertJsonSchemaToGemini(schemaWithRef);
  const props = converted.properties as Record<string, Record<string, unknown>>;

  assert.equal(props.entity.type, "string");
  assert.equal(props.entity.nullable, true);
  assert.equal(props.entity.description, "Nullable entity ID");
});

test("convertJsonSchemaToGemini converts real protocol schemas", async () => {
  const schemaDir = path.resolve(import.meta.dirname, "../../../protocol/schemas");
  const files = await fs.readdir(schemaDir);

  let testedCount = 0;
  for (const file of files) {
    if (!file.endsWith(".json")) continue;
    const content = await fs.readFile(path.join(schemaDir, file), "utf8");
    const parsed = JSON.parse(content);
    if (parsed.definitions) {
      for (const [defName, defSchema] of Object.entries(parsed.definitions)) {
        const converted = convertJsonSchemaToGemini(parsed, defSchema as Record<string, unknown>);
        assert.ok(converted, `Converted schema for ${file} -> ${defName} should exist`);
        assert.equal((converted as Record<string, unknown>).additionalProperties, undefined);
        assert.equal((converted as Record<string, unknown>).$schema, undefined);
        testedCount++;
      }
    }
  }

  assert.ok(testedCount > 20, `Tested ${testedCount} real protocol schemas`);
});
