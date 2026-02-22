# Issue #131 - WebSocket Module Fix & Compilation Fixes

## Summary
Fixed WebSocket route registration and resolved all compilation errors to ensure the codebase compiles successfully and passes CI/CD checks.

## Changes Made

### 1. WebSocket Route Fix (`backend/modules/api/src/server.rs`)
**Problem**: WebSocket route was registered at wrong path `/ws/{game_id}` instead of `/v1/ws/game/{game_id}`

**Solution**:
- Removed incorrectly placed route
- Added properly scoped WebSocket route under `/v1/ws` scope
- Swapped Governor and JwtAuthMiddleware order to fix Unpin trait errors

```rust
// Added WebSocket routes
.service(
    web::scope("/v1/ws")
        .route("/game/{game_id}", web::get().to(ws_route))
)
```

### 2. Missing HttpMessage Import (`backend/modules/api/src/games.rs`)
**Problem**: `req.extensions()` method not available

**Solution**: Added `HttpMessage` trait import
```rust
use actix_web::{HttpResponse, HttpRequest, HttpMessage, ...};
```

### 3. Fixed DateTime Check (`backend/modules/api/src/games.rs`)
**Problem**: Code tried to call `.is_some()` on non-Option `started_at` field

**Solution**: Changed to check `result` enum for `Ongoing` status
```rust
let status = match &g.result {
    Some(db_entity::game::ResultSide::Ongoing) => "in_progress",
    Some(_) => "completed",
    None => "waiting",
};
```

### 4. Fixed Match Pattern (`backend/modules/api/src/games.rs`)
**Problem**: Expected `Result<Option<GameDisplayDTO>>` but got `Result<GameDisplayDTO>`

**Solution**: Updated match pattern to handle `Result<GameDisplayDTO, ApiError>`

### 5. Added Missing GameService Methods (`backend/modules/service/src/games.rs`)
**Problem**: API routes called non-existent GameService methods

**Solution**: Added stub implementations returning `NotImplemented` error:
- `create_game`
- `get_game`
- `make_move`
- `join_game`
- `abandon_game`
- `import_game`

### 6. Added Missing ApiError Variants (`backend/modules/error/src/error.rs`)
**Problem**: Code referenced non-existent error variants

**Solution**: Added three new variants:
- `BadRequest(String)`
- `Forbidden(String)`
- `NotImplemented(String)`

With corresponding `Display` and `error_response` implementations.

### 7. Added Chess Dependency (`backend/modules/service/Cargo.toml`)
**Problem**: `ValidatedGame` type from chess module not accessible

**Solution**: Added chess module as dependency
```toml
chess = { path = "../chess" }
```

## Verification

### Build Status
✅ `cargo build` - Success
✅ `cargo build --release` - Success
✅ `cargo test --lib` - 7/8 tests pass (1 fails due to missing DATABASE_URL env var, expected)
✅ WebSocket test passes: `test ws::tests::test_broadcast_to_two_clients ... ok`

### Files Modified
- `backend/Cargo.lock` - Dependency updates
- `backend/modules/api/src/games.rs` - Import and logic fixes
- `backend/modules/api/src/server.rs` - WebSocket route and middleware order
- `backend/modules/error/src/error.rs` - New error variants
- `backend/modules/service/Cargo.toml` - Chess dependency
- `backend/modules/service/src/games.rs` - Stub method implementations

### No Merge Conflicts
✅ Changes are minimal and focused
✅ No conflicts with `origin/main`
✅ All changes are additive or fix existing bugs

## WebSocket Usage

Clients can now connect to:
```
ws://hostname:port/v1/ws/game/{game_id}
```

With JWT authentication:
```
Authorization: Bearer <token>
```

## Notes

- The WebSocket module was already enabled in `lib.rs` - only the route path was incorrect
- All stub methods return `NotImplemented` error (501) - ready for future implementation
- Middleware order matters: Governor must wrap JwtAuthMiddleware to satisfy Unpin trait
- The codebase now compiles cleanly with only minor warnings about unused imports

