//! Always-on native world process: authoritative `World`, tick loop, snapshots.

use std::sync::{Arc, Mutex};

use terrarium_kernel::{compile_text, CellId, Mass, World};
use tokio::sync::broadcast;

pub const MS_PER_TICK: u64 = 50;

pub const DEMO_WANDER: &str = r#"# wander — fixed thrust loop (deterministic)
thrust 50 20
sleep
thrust -40 45
sleep
thrust -30 -55
sleep
thrust 55 -15
sleep
jump 0
"#;

pub const DEMO_CHASE: &str = r#"# chase — sense nearest body, thrust toward it
sense
jnz 0 4
sleep
jump 0
thrust_toward 70
sleep
jump 0
"#;

pub const DEMO_SIT: &str = r#"# sit — sleep is free
sleep
jump 0
"#;

pub struct WorldHost {
    inner: Mutex<World>,
}

impl WorldHost {
    pub fn new() -> Self {
        let host = Self {
            inner: Mutex::new(World::new()),
        };
        host.reset();
        host
    }

    pub fn reset(&self) {
        let mut world = self.inner.lock().expect("world lock");
        seed_world(&mut world);
    }

    pub fn tick(&self) {
        if let Ok(mut world) = self.inner.lock() {
            world.tick();
        }
    }

    pub fn spawn_cell(
        &self,
        mass: u64,
        x: i32,
        y: i32,
        program: Option<&str>,
    ) -> Result<u64, String> {
        let mut world = self.inner.lock().map_err(|_| "world lock poisoned".to_string())?;
        let cell_id = world
            .spawn_cell_at(Mass::new(mass), x, y)
            .map_err(|e| e.to_string())?;
        if let Some(src) = program {
            if !src.trim().is_empty() {
                let prog = compile_text(src).map_err(|e| e.to_string())?;
                world
                    .set_program(cell_id, prog)
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(cell_id.get())
    }

    pub fn set_program(&self, cell_id: u64, source: &str) -> Result<(), String> {
        let program = compile_text(source).map_err(|e| e.to_string())?;
        self.inner
            .lock()
            .map_err(|_| "world lock poisoned".to_string())?
            .set_program(CellId::new(cell_id), program)
            .map_err(|e| e.to_string())
    }

    pub fn snapshot_json(&self) -> Result<String, String> {
        let world = self
            .inner
            .lock()
            .map_err(|_| "world lock poisoned".to_string())?;
        Ok(snapshot_to_json(&world.snapshot()))
    }
}

pub fn seed_world(world: &mut World) {
    *world = World::new();
    let a = world
        .spawn_cell_at(Mass::new(5000), -120_000, -80_000)
        .expect("spawn a");
    let b = world
        .spawn_cell_at(Mass::new(4000), 100_000, 60_000)
        .expect("spawn b");
    let c = world
        .spawn_cell_at(Mass::new(3500), -40_000, 140_000)
        .expect("spawn c");
    world
        .set_program(a, compile_text(DEMO_WANDER).expect("wander"))
        .expect("program a");
    world
        .set_program(b, compile_text(DEMO_CHASE).expect("chase"))
        .expect("program b");
    world
        .set_program(c, compile_text(DEMO_SIT).expect("sit"))
        .expect("program c");
    let dumper = world
        .spawn_cell_at(Mass::new(800), 80_000, -100_000)
        .expect("spawn dumper");
    world
        .dump_matter(dumper, Mass::new(400))
        .expect("dump crumb");
    world
        .set_program(dumper, compile_text(DEMO_SIT).expect("sit"))
        .expect("program dumper");
}

pub fn snapshot_to_json(s: &terrarium_kernel::WorldSnapshot) -> String {
    serde_json::json!({
        "tick": s.tick,
        "total_mass": s.total_mass.get(),
        "house_burned": s.house_burned.get(),
        "spawned_mass": s.spawned_mass.get(),
        "width": s.width,
        "height": s.height,
        "cells": s.cells.iter().map(|c| serde_json::json!({
            "id": c.id.get(),
            "mass": c.mass.get(),
            "x": c.x,
            "y": c.y,
        })).collect::<Vec<_>>(),
        "inert": s.inert.iter().map(|n| serde_json::json!({
            "id": n.id.get(),
            "mass": n.mass.get(),
            "x": n.x,
            "y": n.y,
        })).collect::<Vec<_>>(),
    })
    .to_string()
}

pub fn wrap_snapshot(raw_object: &str) -> String {
    format!(r#"{{"type":"snapshot","world":{raw_object}}}"#)
}

pub fn spawn_tick_loop(host: Arc<WorldHost>, snapshots: broadcast::Sender<String>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(MS_PER_TICK));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            host.tick();
            if snapshots.receiver_count() == 0 {
                continue;
            }
            if let Ok(json) = host.snapshot_json() {
                let _ = snapshots.send(wrap_snapshot(&json));
            }
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_and_tick_advances() {
        let host = WorldHost::new();
        let before = host.snapshot_json().unwrap();
        let tick_before: u64 = serde_json::from_str::<serde_json::Value>(&before)
            .unwrap()
            .get("tick")
            .and_then(|v| v.as_u64())
            .unwrap();
        host.tick();
        let after = host.snapshot_json().unwrap();
        let tick_after: u64 = serde_json::from_str::<serde_json::Value>(&after)
            .unwrap()
            .get("tick")
            .and_then(|v| v.as_u64())
            .unwrap();
        assert_eq!(tick_after, tick_before + 1);
    }

    #[test]
    fn spawn_increases_spawned_mass() {
        let host = WorldHost::new();
        let before: u64 = serde_json::from_str::<serde_json::Value>(&host.snapshot_json().unwrap())
            .unwrap()
            .get("spawned_mass")
            .and_then(|v| v.as_u64())
            .unwrap();
        host.spawn_cell(100, 0, 0, None).unwrap();
        let after: u64 = serde_json::from_str::<serde_json::Value>(&host.snapshot_json().unwrap())
            .unwrap()
            .get("spawned_mass")
            .and_then(|v| v.as_u64())
            .unwrap();
        assert_eq!(after, before + 100);
    }
}
