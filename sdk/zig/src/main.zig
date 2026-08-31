const terrarium = @import("terrarium.zig");

/// Called once per sim tick. One action per tick — see docs/bytecode.md.
export fn tick() void {
    // Step forward; blocked cells are a no-op on the host side.
    _ = terrarium.move_forward();
    terrarium.sleep_host();
}
