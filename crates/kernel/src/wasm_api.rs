//! WASM surface for the skin. Same kernel, camera talks through this.

use wasm_bindgen::prelude::*;

use crate::program::{compile_text, decode_program, encode_program};
use crate::world::{Mass, World, WORLD_RADIUS};

/// JS-facing world handle.
#[wasm_bindgen]
pub struct JsWorld {
    inner: World,
}

#[wasm_bindgen]
impl JsWorld {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsWorld {
        JsWorld {
            inner: World::with_radius(WORLD_RADIUS),
        }
    }

    #[wasm_bindgen(js_name = worldRadius)]
    pub fn world_radius(&self) -> i32 {
        self.inner.radius()
    }

    #[wasm_bindgen(js_name = totalMass)]
    pub fn total_mass(&self) -> f64 {
        self.inner.total_mass().get() as f64
    }

    #[wasm_bindgen(js_name = houseBurned)]
    pub fn house_burned(&self) -> f64 {
        self.inner.house_burned().get() as f64
    }

    #[wasm_bindgen(js_name = spawnedMass)]
    pub fn spawned_mass(&self) -> f64 {
        self.inner.spawned_mass().get() as f64
    }

    #[wasm_bindgen(js_name = tickCount)]
    pub fn tick_count(&self) -> f64 {
        self.inner.tick_count() as f64
    }

    /// Spawn a cell. Returns cell id, or throws on error.
    #[wasm_bindgen(js_name = spawnCell)]
    pub fn spawn_cell(&mut self, mass: f64, x: i32, y: i32) -> Result<u32, JsValue> {
        let m = Mass::new(mass as u64);
        self.inner
            .spawn_cell_at(m, x, y)
            .map(|id| id.get() as u32)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Install a text program on a cell.
    #[wasm_bindgen(js_name = setProgramText)]
    pub fn set_program_text(&mut self, cell_id: u32, src: &str) -> Result<(), JsValue> {
        let program = compile_text(src).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner
            .set_program(crate::world::CellId::from_raw(cell_id as u64), program)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Install raw bytecode on a cell.
    #[wasm_bindgen(js_name = setProgramBytes)]
    pub fn set_program_bytes(&mut self, cell_id: u32, bytes: &[u8]) -> Result<(), JsValue> {
        let program = decode_program(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.inner
            .set_program(crate::world::CellId::from_raw(cell_id as u64), program)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Compile text → bytecode (for skin tooling / demos).
    #[wasm_bindgen(js_name = compileProgram)]
    pub fn compile_program(src: &str) -> Result<Vec<u8>, JsValue> {
        let program = compile_text(src).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(encode_program(&program))
    }

    /// Dump inert mass from a cell (JSON-friendly). Conserves total_mass.
    #[wasm_bindgen(js_name = dumpMatter)]
    pub fn dump_matter(&mut self, cell_id: u32, amount: f64) -> Result<u32, JsValue> {
        self.inner
            .dump_matter(
                crate::world::CellId::from_raw(cell_id as u64),
                Mass::new(amount as u64),
            )
            .map(|id| id.get() as u32)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn tick(&mut self) {
        self.inner.tick();
    }

    /// JSON snapshot for the canvas.
    pub fn snapshot(&self) -> String {
        let s = self.inner.snapshot();
        let mut out = String::from("{\"tick\":");
        out.push_str(&s.tick.to_string());
        out.push_str(",\"total_mass\":");
        out.push_str(&s.total_mass.get().to_string());
        out.push_str(",\"house_burned\":");
        out.push_str(&s.house_burned.get().to_string());
        out.push_str(",\"spawned_mass\":");
        out.push_str(&s.spawned_mass.get().to_string());
        out.push_str(",\"radius\":");
        out.push_str(&s.radius.to_string());
        out.push_str(",\"cells\":[");
        for (i, c) in s.cells.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"id\":{},\"mass\":{},\"x\":{},\"y\":{},\"vx\":{},\"vy\":{},\"pc\":{},\"halted\":{}}}",
                c.id.get(),
                c.mass.get(),
                c.x,
                c.y,
                c.vx,
                c.vy,
                c.pc,
                c.halted
            ));
        }
        out.push_str("],\"inert\":[");
        for (i, n) in s.inert.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"id\":{},\"mass\":{},\"x\":{},\"y\":{}}}",
                n.id.get(),
                n.mass.get(),
                n.x,
                n.y
            ));
        }
        out.push_str("]}");
        out
    }
}

impl Default for JsWorld {
    fn default() -> Self {
        Self::new()
    }
}
