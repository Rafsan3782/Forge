import test from "node:test";
import assert from "node:assert/strict";
import http from "node:http";
import { WebSocketServer, WebSocket } from "ws";
import { EngineMcpClient } from "./engine_mcp.ts";
import { ToolRegistry } from "./registry.ts";
import { formatScreenshotAsImagePart, calculateDownscaledDimensions } from "./vision.ts";

test("calculateDownscaledDimensions scales exceeding dimensions preserving aspect ratio", () => {
  const landscape = calculateDownscaledDimensions(1920, 1080, 1024);
  assert.equal(landscape.scaled, true);
  assert.equal(landscape.width, 1024);
  assert.equal(landscape.height, 576);

  const portrait = calculateDownscaledDimensions(1080, 1920, 1024);
  assert.equal(portrait.scaled, true);
  assert.equal(portrait.height, 1024);
  assert.equal(portrait.width, 576);

  const small = calculateDownscaledDimensions(800, 600, 1024);
  assert.equal(small.scaled, false);
  assert.equal(small.width, 800);
  assert.equal(small.height, 600);
});

test("formatScreenshotAsImagePart formats base64 and strips header data URL", () => {
  const part = formatScreenshotAsImagePart("data:image/png;base64,iVBORw0KGgoAAAANSUhEUg==");
  assert.equal(part.inline_data.mime_type, "image/png");
  assert.equal(part.inline_data.data, "iVBORw0KGgoAAAANSUhEUg==");
});

test("ToolRegistry collision check and provider registration", () => {
  const registry = new ToolRegistry();

  registry.registerTool({
    descriptor: {
      name: "test.tool",
      description: "A test tool",
      category: "read",
      risk_level: "safe",
      input_schema: { type: "object" },
    },
    metadata: { category: "read", risk: "safe" },
    execute: async () => ({ status: "ok" }),
  });

  // Attempt duplicate registration
  assert.throws(
    () => {
      registry.registerTool({
        descriptor: {
          name: "test.tool",
          description: "Colliding tool",
          category: "read",
          risk_level: "safe",
          input_schema: { type: "object" },
        },
        metadata: { category: "read", risk: "safe" },
        execute: async () => ({ status: "error" }),
      });
    },
    (err: unknown) => {
      return (
        err instanceof Error &&
        ("code" in err ? (err as { code: string }).code === "TOOL_COLLISION" : false)
      );
    }
  );
});

test("EngineMcpClient with fake WebSocket bridge server", async () => {
  const server = http.createServer();
  const wss = new WebSocketServer({ server });

  const token = "test_auth_token_xyz";

  wss.on("connection", (ws: WebSocket, req) => {
    const url = new URL(req.url ?? "", "http://127.0.0.1");
    if (url.searchParams.get("token") !== token) {
      ws.close(4401, "Unauthorized");
      return;
    }

    ws.on("message", (raw) => {
      const parsed = JSON.parse(raw.toString("utf8"));
      const { id, method } = parsed;

      if (method === "tools/list") {
        ws.send(
          JSON.stringify({
            jsonrpc: "2.0",
            id,
            result: {
              tools: [
                {
                  name: "scene.get_tree",
                  description: "Get scene tree",
                  input_schema: { type: "object" },
                  category: "read",
                  risk_level: "safe",
                },
                {
                  name: "game.screenshot",
                  description: "Capture game screenshot",
                  input_schema: { type: "object" },
                  category: "read",
                  risk_level: "safe",
                },
              ],
            },
          })
        );
      } else if (method === "scene.get_tree") {
        ws.send(
          JSON.stringify({
            jsonrpc: "2.0",
            id,
            result: { root_entities: [{ id: "ent_1", name: "Root" }] },
          })
        );
      } else if (method === "game.screenshot") {
        ws.send(
          JSON.stringify({
            jsonrpc: "2.0",
            id,
            result: {
              data_base64: "iVBORw0KGgoAAAANSUhEUg==",
              width: 800,
              height: 600,
              format: "png",
            },
          })
        );
      }
    });
  });

  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", () => resolve()));
  const address = server.address() as { port: number };

  try {
    const engineMcp = new EngineMcpClient({
      port: address.port,
      token,
      autoReconnect: false,
    });

    await engineMcp.connect();
    const tools = engineMcp.getTools();
    assert.equal(tools.length, 2);
    assert.ok(tools.some((t) => t.descriptor.name === "scene.get_tree"));
    assert.ok(tools.some((t) => t.descriptor.name === "game.screenshot"));

    const treeRes = (await engineMcp.executeTool("scene.get_tree", {})) as {
      root_entities: Array<{ id: string; name: string }>;
    };
    assert.ok(treeRes.root_entities);
    assert.equal(treeRes.root_entities[0].id, "ent_1");

    const shotRes = (await engineMcp.executeTool("game.screenshot", {})) as {
      image_part: { inline_data: { data: string } };
    };
    assert.ok(shotRes.image_part);
    assert.equal(shotRes.image_part.inline_data.data, "iVBORw0KGgoAAAANSUhEUg==");

    engineMcp.disconnect();
  } finally {
    wss.close();
    server.close();
  }
});

test("ToolProvider contract tests against ToolRegistry", async () => {
  const registry = new ToolRegistry();

  registry.registerTool({
    descriptor: {
      name: "scene.get_tree",
      description: "Get scene entity hierarchy",
      category: "read",
      risk_level: "safe",
      input_schema: { type: "object" },
    },
    metadata: { category: "read", risk: "safe" },
    execute: async () => ({ root_entities: [] }),
  });

  registry.registerTool({
    descriptor: {
      name: "script.write",
      description: "Write script file",
      category: "write",
      risk_level: "low",
      input_schema: { type: "object" },
    },
    metadata: { category: "write", risk: "low" },
    execute: async (args) => ({ path: args.path, written: true }),
  });

  const list = await registry.listTools();
  assert.ok(list.length >= 2);
  assert.ok(list.some((t) => t.name === "scene.get_tree"));

  const res = await registry.executeTool("script.write", {
    path: "scripts/test.ts",
    content: "export const x = 1;",
  });

  assert.equal(res.is_error, false);
  assert.equal((res.content as { path: string }).path, "scripts/test.ts");
});
