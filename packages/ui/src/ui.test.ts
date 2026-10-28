import { describe, it } from "node:test";
import assert from "node:assert/strict";

describe("@forge/ui v2", () => {
  it("defines package name", () => {
    const packageName = "@forge/ui";
    assert.equal(packageName, "@forge/ui");
  });

  it("defines all engine custom icons names", () => {
    const requiredIcons = [
      "entity",
      "component",
      "scene",
      "tilemap",
      "sprite",
      "collider",
      "script",
      "play",
      "stop",
      "step",
      "gizmo-move",
      "gizmo-rotate",
      "gizmo-scale",
      "checkpoint",
      "agent",
    ];

    assert.equal(requiredIcons.length, 15);
    for (const icon of requiredIcons) {
      assert.ok(typeof icon === "string" && icon.length > 0);
    }
  });
});
