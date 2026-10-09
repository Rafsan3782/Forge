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
