// Reference module for local builds — editor source is written to user.rs as-is.
// Compile worker injects prelude and wraps the body in pub fn main().

pub fn main() {
    loop {
        let _ = move_forward();
    }
}
