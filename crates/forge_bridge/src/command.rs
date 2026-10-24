//! Bridge commands transferred between WebSocket server tasks and the game engine loop.

use forge_protocol::{
    EnginePingParams, EnginePingResult, GameScreenshotParams, GameScreenshotResult,
    GameSimulateInputParams, GameSimulateInputResult, ProjectStopParams, ProjectStopResult,
    SceneCreateEntityParams, SceneCreateEntityResult, SceneDeleteEntityParams,
    SceneDeleteEntityResult, SceneGetTreeParams, SceneGetTreeResult, SceneSetComponentParams,
    SceneSetComponentResult,
};
use tokio::sync::{mpsc, oneshot};

/// Individual commands sent from the bridge network thread to the engine thread.
#[derive(Debug)]
pub enum BridgeCommand {
    /// Ping request to test engine responsiveness.
    EnginePing {
        /// Ping parameters.
        params: EnginePingParams,
        /// Channel to return the ping result.
        responder: oneshot::Sender<Result<EnginePingResult, String>>,
    },
    /// Query the scene entity tree.
    SceneGetTree {
        /// Scene query parameters.
        params: SceneGetTreeParams,
        /// Channel to return the hierarchy tree.
        responder: oneshot::Sender<Result<SceneGetTreeResult, String>>,
    },
    /// Create a new entity in the world.
    SceneCreateEntity {
        /// Entity creation parameters.
        params: SceneCreateEntityParams,
        /// Channel to return the created entity id.
        responder: oneshot::Sender<Result<SceneCreateEntityResult, String>>,
    },
    /// Set or update a component on an entity.
    SceneSetComponent {
        /// Component payload parameters.
        params: SceneSetComponentParams,
        /// Channel to return operation status.
        responder: oneshot::Sender<Result<SceneSetComponentResult, String>>,
    },
    /// Delete an entity and its descendants.
    SceneDeleteEntity {
        /// Entity deletion parameters.
        params: SceneDeleteEntityParams,
        /// Channel to return operation status.
        responder: oneshot::Sender<Result<SceneDeleteEntityResult, String>>,
    },
    /// Capture a viewport screenshot.
    GameScreenshot {
        /// Screenshot configuration parameters.
        params: GameScreenshotParams,
        /// Channel to return the screenshot result (base64 or saved file path).
        responder: oneshot::Sender<Result<GameScreenshotResult, String>>,
    },
    /// Simulate input events on the engine thread.
    GameSimulateInput {
        /// Input simulation parameters.
        params: GameSimulateInputParams,
        /// Channel to return operation result.
        responder: oneshot::Sender<Result<GameSimulateInputResult, String>>,
    },
    /// Gracefully stop the game runtime session.
    ProjectStop {
        /// Stop request parameters.
        params: ProjectStopParams,
        /// Channel to return shutdown confirmation.
        responder: oneshot::Sender<Result<ProjectStopResult, String>>,
    },
    /// List assets known to the asset database.
    AssetList {
        /// Asset list filter parameters.
        params: forge_protocol::AssetListParams,
        /// Channel to return list of assets.
        responder: oneshot::Sender<Result<forge_protocol::AssetListResult, String>>,
    },
    /// Get metadata and details for a specific asset ID.
    AssetGetInfo {
        /// Asset get info parameters.
        params: forge_protocol::AssetGetInfoParams,
        /// Channel to return asset info.
        responder: oneshot::Sender<Result<forge_protocol::AssetGetInfoResult, String>>,
    },
    /// Import an asset file into the project.
    AssetImport {
        /// Asset import parameters.
        params: forge_protocol::AssetImportParams,
        /// Channel to return import result.
        responder: oneshot::Sender<Result<forge_protocol::AssetImportResult, String>>,
    },
    /// Perform a physics raycast query in the active simulation world.
    PhysicsRaycast {
        /// Raycast parameters.
        params: forge_protocol::PhysicsRaycastParams,
        /// Channel to return raycast hit result.
        responder: oneshot::Sender<Result<forge_protocol::PhysicsRaycastResult, String>>,
    },
    /// Query tile ID at specific tilemap coordinate.
    TilemapGet {
        /// Parameters specifying entity, layer, and coordinates.
        params: forge_protocol::TilemapGetParams,
        /// Channel to return tile query result.
        responder: oneshot::Sender<Result<forge_protocol::TilemapGetResult, String>>,
    },
    /// Set a tile ID at specific tilemap coordinate.
    TilemapSetTile {
        /// Parameters specifying entity, layer, coordinates, and tile ID.
        params: forge_protocol::TilemapSetTileParams,
        /// Channel to return previous tile ID and affected chunk.
        responder: oneshot::Sender<Result<forge_protocol::TilemapSetTileResult, String>>,
    },
    /// Fill a rectangle of tiles on a tilemap.
    TilemapFillRect {
        /// Parameters specifying entity, layer, bounds, and tile ID.
        params: forge_protocol::TilemapFillRectParams,
        /// Channel to return modified tile and chunk counts.
        responder: oneshot::Sender<Result<forge_protocol::TilemapFillRectResult, String>>,
    },
    /// Clear all tiles or a specific layer from a tilemap.
    TilemapClear {
        /// Parameters specifying entity and optional layer.
        params: forge_protocol::TilemapClearParams,
        /// Channel to return number of cleared chunks.
        responder: oneshot::Sender<Result<forge_protocol::TilemapClearResult, String>>,
    },
    /// Play a sound or music track.
    AudioPlay {
        /// Playback parameters.
        params: forge_protocol::AudioPlayParams,
        /// Channel to return playback ID.
        responder: oneshot::Sender<Result<forge_protocol::AudioPlayResult, String>>,
    },
    /// Stop playback of an active sound or all sounds on a bus.
    AudioStop {
        /// Stop parameters.
        params: forge_protocol::AudioStopParams,
        /// Channel to return stop confirmation.
        responder: oneshot::Sender<Result<forge_protocol::AudioStopResult, String>>,
    },
    /// Set the volume multiplier of an audio bus.
    AudioSetBusVolume {
        /// Bus volume parameters.
        params: forge_protocol::AudioSetBusVolumeParams,
        /// Channel to return updated volume confirmation.
        responder: oneshot::Sender<Result<forge_protocol::AudioSetBusVolumeResult, String>>,
    },
    /// Runs TypeScript typecheck on a project scripts folder.
    ScriptCheck {
        /// Typecheck parameters.
        params: forge_protocol::ScriptCheckParams,
        /// Channel to return check result.
        responder: oneshot::Sender<Result<forge_protocol::ScriptCheckResult, String>>,
    },
    /// Lists TypeScript script files in a project.
    ScriptList {
        /// List parameters.
        params: forge_protocol::ScriptListParams,
        /// Channel to return list of script paths.
        responder: oneshot::Sender<Result<forge_protocol::ScriptListResult, String>>,
    },
    /// Reads the source text of a script file.
    ScriptRead {
        /// Read parameters.
        params: forge_protocol::ScriptReadParams,
        /// Channel to return script content.
        responder: oneshot::Sender<Result<forge_protocol::ScriptReadResult, String>>,
    },
    /// Writes content to a script file, validating SWC TypeScript syntax beforehand.
    ScriptWrite {
        /// Write parameters.
        params: forge_protocol::ScriptWriteParams,
        /// Channel to return write result.
        responder: oneshot::Sender<Result<forge_protocol::ScriptWriteResult, String>>,
    },
    /// Undo the most recent scene command.
    EditUndo {
        /// Undo parameters.
        params: forge_protocol::EditUndoParams,
        /// Channel to return undo result.
        responder: oneshot::Sender<Result<forge_protocol::EditUndoResult, String>>,
    },
    /// Redo the previously undone scene command.
    EditRedo {
        /// Redo parameters.
        params: forge_protocol::EditRedoParams,
        /// Channel to return redo result.
        responder: oneshot::Sender<Result<forge_protocol::EditRedoResult, String>>,
    },
    /// Get the history of scene commands.
    EditGetHistory {
        /// History query parameters.
        params: forge_protocol::EditGetHistoryParams,
        /// Channel to return history result.
        responder: oneshot::Sender<Result<forge_protocol::EditGetHistoryResult, String>>,
    },
    /// Jump to a specific point in the command history stack.
    EditJumpHistory {
        /// Target history ID.
        params: forge_protocol::EditJumpHistoryParams,
        /// Channel to return jump result.
        responder: oneshot::Sender<Result<forge_protocol::EditJumpHistoryResult, String>>,
    },
    /// Get dynamic component schemas with editor hints for the inspector.
    SceneGetComponentSchemas {
        /// Schema query parameters.
        params: forge_protocol::SceneGetComponentSchemasParams,
        /// Channel to return component schemas.
        responder: oneshot::Sender<Result<forge_protocol::SceneGetComponentSchemasResult, String>>,
    },
    /// Save the active scene to disk.
    SceneSave {
        /// Scene save parameters.
        params: forge_protocol::SceneSaveParams,
        /// Channel to return save result.
        responder: oneshot::Sender<Result<forge_protocol::SceneSaveResult, String>>,
    },
    /// Check whether the active scene has unsaved changes.
    SceneGetDirty {
        /// Dirty query parameters.
        params: forge_protocol::SceneGetDirtyParams,
        /// Channel to return dirty state.
        responder: oneshot::Sender<Result<forge_protocol::SceneGetDirtyResult, String>>,
    },
    /// Reparent an entity to a new parent or root.
    SceneReparentEntity {
        /// Reparent parameters.
        params: forge_protocol::SceneReparentEntityParams,
        /// Channel to return reparent result.
        responder: oneshot::Sender<Result<forge_protocol::SceneReparentEntityResult, String>>,
    },
    /// Duplicate an existing entity and its components.
    SceneDuplicateEntity {
        /// Duplicate parameters.
        params: forge_protocol::SceneDuplicateEntityParams,
        /// Channel to return duplicated entity info.
        responder: oneshot::Sender<Result<forge_protocol::SceneDuplicateEntityResult, String>>,
    },
    /// Remove a component from an entity.
    SceneRemoveComponent {
        /// Remove component parameters.
        params: forge_protocol::SceneRemoveComponentParams,
        /// Channel to return remove result.
        responder: oneshot::Sender<Result<forge_protocol::SceneRemoveComponentResult, String>>,
    },
}

/// Sender side held by the bridge server to dispatch commands to the engine loop.
#[derive(Clone, Debug)]
pub struct BridgeCommandSender {
    sender: mpsc::UnboundedSender<BridgeCommand>,
}

impl BridgeCommandSender {
    /// Creates a new sender wrapping an unbounded MPSC channel.
    pub fn new(sender: mpsc::UnboundedSender<BridgeCommand>) -> Self {
        Self { sender }
    }

    /// Dispatches a command to the engine thread.
    pub fn send(&self, cmd: BridgeCommand) -> Result<(), mpsc::error::SendError<BridgeCommand>> {
        self.sender.send(cmd)
    }
}

/// Receiver side held by the engine runtime loop, drained without blocking between frames.
#[derive(Debug)]
pub struct BridgeCommandReceiver {
    receiver: mpsc::UnboundedReceiver<BridgeCommand>,
}

impl BridgeCommandReceiver {
    /// Creates a new receiver wrapping an unbounded MPSC channel.
    pub fn new(receiver: mpsc::UnboundedReceiver<BridgeCommand>) -> Self {
        Self { receiver }
    }

    /// Attempts to receive a pending command non-blockingly.
    pub fn try_recv(&mut self) -> Result<BridgeCommand, mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }
}

/// Creates a paired sender and receiver for bridge commands.
pub fn create_bridge_channel() -> (BridgeCommandSender, BridgeCommandReceiver) {
    let (tx, rx) = mpsc::unbounded_channel();
    (BridgeCommandSender::new(tx), BridgeCommandReceiver::new(rx))
}
