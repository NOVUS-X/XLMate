## Pull Request Summary

### Issue #131: Enable WebSocket Module for Real-Time Game Updates

#### Problem
The WebSocket module was fully implemented but:
1. Route registered at wrong path (`/ws/{game_id}` instead of `/v1/ws/game/{game_id}`)
2. Multiple compilation errors prevented the codebase from building
3. Missing GameService method implementations
4. Missing ApiError variants

#### Solution
Fixed all compilation errors and enabled WebSocket functionality:

**WebSocket Route** (`server.rs`)
- Moved route to correct path: `/v1/ws/game/{game_id}`
- Fixed middleware order (Governor → JwtAuthMiddleware) to resolve Unpin errors

**Compilation Fixes**
- Added missing `HttpMessage` import in `games.rs`
- Fixed `started_at` check to use `ResultSide::Ongoing` enum
- Added stub implementations for GameService methods
- Added `BadRequest`, `Forbidden`, `NotImplemented` error variants
- Added chess module dependency to service

#### Testing
✅ All builds pass: `cargo build` and `cargo build --release`
✅ WebSocket test passes: `test ws::tests::test_broadcast_to_two_clients`
✅ 7/8 unit tests pass (1 requires DATABASE_URL env var)

#### Files Changed
- `backend/modules/api/src/server.rs` - WebSocket route & middleware
- `backend/modules/api/src/games.rs` - Import & logic fixes
- `backend/modules/service/src/games.rs` - Stub method implementations
- `backend/modules/error/src/error.rs` - New error variants
- `backend/modules/service/Cargo.toml` - Chess dependency
- `backend/Cargo.lock` - Dependency updates

#### Breaking Changes
None - all changes are additive or fix existing bugs

#### Notes
- Stub methods return 501 Not Implemented - ready for future implementation
- No merge conflicts with main branch
- Codebase now compiles cleanly
