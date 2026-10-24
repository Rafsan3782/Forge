import { describe, it, expect, beforeEach } from "vitest";
import { useEditStore } from "../stores/editStore";
import { useSceneStore } from "../stores/sceneStore";

describe("EditStore and SceneStore editing", () => {
  beforeEach(() => {
    useEditStore.setState({
      undoStack: [],
      redoStack: [],
      canUndo: false,
      canRedo: false,
      isDirty: false,
      scenePath: null,
      componentSchemas: {
        Transform2D: {
          name: "Transform2D",
          display_name: "Transform 2D",
          fields: [
            {
              name: "position",
              field_type: "vector2",
              hint: { widget: "vector2", step: 0.5 },
            },
          ],
        },
      },
    });

    useSceneStore.setState({
      entities: [
        {
          id: "entity_1",
          name: "Hero",
          components: ["Name", "Transform2D"],
          children: [],
        },
      ],
      selectedEntityId: "entity_1",
      selectedEntityIds: ["entity_1"],
      selectedEntity: {
        id: "entity_1",
        name: "Hero",
        components: ["Name", "Transform2D"],
        children: [],
      },
    });
  });

  it("selects entity and updates selection arrays", () => {
    const sceneStore = useSceneStore.getState();
    sceneStore.selectEntity("entity_1");
    expect(useSceneStore.getState().selectedEntityId).toBe("entity_1");
    expect(useSceneStore.getState().selectedEntityIds).toEqual(["entity_1"]);
  });

  it("supports multi-selection toggle", () => {
    const sceneStore = useSceneStore.getState();
    sceneStore.selectEntity("entity_2", true);
    expect(useSceneStore.getState().selectedEntityIds).toContain("entity_1");
    expect(useSceneStore.getState().selectedEntityIds).toContain("entity_2");

    sceneStore.selectEntity("entity_1", true);
    expect(useSceneStore.getState().selectedEntityIds).not.toContain("entity_1");
    expect(useSceneStore.getState().selectedEntityIds).toContain("entity_2");
  });

  it("manages dirty state tracking", () => {
    const editStore = useEditStore.getState();
    expect(editStore.isDirty).toBe(false);

    editStore.setDirty(true);
    expect(useEditStore.getState().isDirty).toBe(true);

    editStore.setDirty(false);
    expect(useEditStore.getState().isDirty).toBe(false);
  });

  it("stores dynamic component schemas", () => {
    const schemas = useEditStore.getState().componentSchemas;
    expect(schemas.Transform2D).toBeDefined();
    expect(schemas.Transform2D.fields[0].hint?.widget).toBe("vector2");
  });
});
