# Architecture & Design Decisions

### 2026-10-26: Kinematic Platformer Controller
- **Decision**: Implement a custom kinematic platformer controller in TypeScript with raycast/AABB ground detection rather than relying on dynamic physics bodies.
- **Why**: Traditional rigidbodies suffer from edge-snagging, friction inconsistencies, and floatiness on tilemap edges; kinematic math guarantees pixel-crisp jumps, coyote time, and instant response.

### 2026-10-26: Tilemap Chunking Architecture
- **Decision**: Represent tilemap terrain in discrete 16x16 tile chunks with run-length string encoding.
- **Why**: Enables efficient spatial streaming, minimal memory footprint in scene files, and seamless rendering via instanced mesh draws.

### 2026-10-26: Sound Bus Routing
- **Decision**: Segregate audio output into `master`, `music`, and `sfx` buses.
- **Why**: Allows players and UI volume sliders to balance ambient chiptunes against action sound effects independently.
