//! Canonical WAT example programs (keep in sync with apps/skin/src/lib/examples.ts).

pub struct ExampleProgram {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub code: &'static str,
}

pub const ALL: &[ExampleProgram] = &[
    ExampleProgram {
        id: "runner",
        name: "Runner",
        description: "Wanders randomly each tick",
        code: RUNNER,
    },
    ExampleProgram {
        id: "prey",
        name: "Prey",
        description: "Flees adjacent threats, otherwise wanders",
        code: PREY,
    },
    ExampleProgram {
        id: "predator",
        name: "Predator",
        description: "Hunts creatures in vision — eats when adjacent, wanders when idle",
        code: PREDATOR,
    },
    ExampleProgram {
        id: "scavenger",
        name: "Scavenger",
        description: "Hunts corpses in vision — eats when adjacent, wanders when idle",
        code: SCAVENGER,
    },
    ExampleProgram {
        id: "colonist",
        name: "Colonist",
        description: "Buds east when energy > 10M (works at default deploy)",
        code: COLONIST,
    },
    ExampleProgram {
        id: "hawk",
        name: "Hawk",
        description: "Chases prey alarms (0x01), otherwise wanders",
        code: HAWK,
    },
    ExampleProgram {
        id: "beacon",
        name: "Beacon",
        description: "Broadcasts heartbeat (0xBE) and wanders",
        code: BEACON,
    },
    ExampleProgram {
        id: "kamikaze",
        name: "Kamikaze",
        description: "Self-destructs below 3M energy",
        code: KAMIKAZE,
    },
    ExampleProgram {
        id: "idle",
        name: "Idle",
        description: "Sleep only — for testing gas cost",
        code: IDLE,
    },
];

pub(crate) const IDLE: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (func (export "tick") (call $sleep))
)
"#;

pub(crate) const RUNNER: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (import "terrarium" "random_byte" (func $rand (result i32)))
  (func (export "tick")
    call $rand
    i32.const 3
    i32.and
    call $move
    drop
    call $sleep)
)
"#;

pub(crate) const BEACON: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (import "terrarium" "signal_broadcast" (func $broadcast (param i32) (result i32)))
  (import "terrarium" "random_byte" (func $rand (result i32)))
  (func (export "tick")
    i32.const 190
    call $broadcast
    drop
    call $rand
    i32.const 3
    i32.and
    call $move
    drop
    call $sleep)
)
"#;

pub(crate) const KAMIKAZE: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "energy" (func $energy (result i64)))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (import "terrarium" "random_byte" (func $rand (result i32)))
  (import "terrarium" "suicide" (func $suicide))
  (func (export "tick")
    call $energy
    i64.const 3000000
    i64.lt_s
    if
      call $suicide
      return
    end
    call $rand
    i32.const 3
    i32.and
    call $move
    drop
    call $sleep)
)
"#;

const COLONIST: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "energy" (func $energy (result i64)))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (import "terrarium" "random_byte" (func $rand (result i32)))
  (import "terrarium" "spawn" (func $spawn (param i32 i32) (result i32)))
  (func (export "tick")
    call $energy
    i64.const 10000000
    i64.le_s
    if
      call $rand
      i32.const 3
      i32.and
      call $move
      drop
      call $sleep
      return
    end
    i32.const 1
    i32.const 1000000
    call $spawn
    drop
    call $sleep)
)
"#;

pub(crate) const SCAVENGER: &str = r#"
(module $strategy_scavenger.wasm
  (type (;0;) (func))
  (type (;1;) (func (result i32)))
  (type (;2;) (func (param i32) (result i32)))
  (type (;3;) (func (param i32 i32) (result i32)))
  (type (;4;) (func (param i32)))
  (import "terrarium" "sleep" (func $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E (;0;) (type 0)))
  (import "terrarium" "random_byte" (func $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E (;1;) (type 1)))
  (import "terrarium" "move" (func $_ZN15strategy_hunter4host4step17h54526a67a501102bE (;2;) (type 2)))
  (import "terrarium" "sense_at" (func $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E (;3;) (type 3)))
  (import "terrarium" "eat" (func $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE (;4;) (type 2)))
  (memory (;0;) 16)
  (global $__stack_pointer (;0;) (mut i32) i32.const 1048576)
  (global (;1;) i32 i32.const 1048576)
  (global (;2;) i32 i32.const 1048576)
  (export "memory" (memory 0))
  (export "tick" (func $tick))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2))
  (func $tick (;5;) (type 0)
    i32.const 3
    call $_ZN15strategy_hunter4tick17h13d0a9efadcd391bE
  )
  (func $_ZN15strategy_hunter4tick17h13d0a9efadcd391bE (;6;) (type 4) (param i32)
    (local i32)
    i32.const 0
    local.set 1
    block ;; label = @1
      block ;; label = @2
        i32.const 0
        i32.const -1
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.eq
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        i32.const 1
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.eq
        br_if 0 (;@2;)
        block ;; label = @3
          i32.const 0
          i32.const 1
          call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
          local.get 0
          i32.ne
          br_if 0 (;@3;)
          i32.const 2
          call $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE
          drop
          call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
          return
        end
        i32.const -1
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 1 (;@1;)
        i32.const 3
        local.set 1
      end
      local.get 1
      call $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE
      drop
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
      return
    end
    i32.const 1
    local.set 1
    block ;; label = @1
      i32.const 1
      i32.const 0
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      i32.const 0
      local.set 1
      i32.const 0
      i32.const -1
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -1
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 1
      local.set 1
      block ;; label = @2
        i32.const 0
        i32.const 1
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 2
        local.set 1
        br 1 (;@1;)
      end
      i32.const 1
      i32.const 1
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      i32.const 1
      local.set 1
      i32.const 1
      i32.const -1
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -1
        i32.const -1
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const -1
        i32.const 1
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      local.set 1
      block ;; label = @2
        i32.const 2
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      i32.const -2
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -2
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 2
      local.set 1
      i32.const 0
      i32.const 2
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const 2
        i32.const 2
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 2
        i32.const -2
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const -2
        i32.const -2
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 3
      local.set 1
      i32.const -2
      i32.const 2
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      i32.const 0
      local.set 1
      block ;; label = @2
        i32.const 3
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      i32.const -3
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -3
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 0
        i32.const 3
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 2
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 3
        i32.const 3
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 3
      local.set 1
      block ;; label = @2
        i32.const 3
        i32.const -3
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const -3
      i32.const -3
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -3
        i32.const 3
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      local.set 1
      block ;; label = @2
        i32.const 4
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      i32.const -4
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -4
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 0
        i32.const 4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 2
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 4
        i32.const 4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 4
        i32.const -4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const -4
        i32.const -4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const -4
        i32.const 4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      local.set 1
      block ;; label = @2
        i32.const 5
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      i32.const -5
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -5
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 2
      local.set 1
      i32.const 0
      i32.const 5
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const 5
        i32.const 5
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 1
      local.set 1
      i32.const 5
      i32.const -5
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -5
        i32.const -5
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 3
      local.set 1
      i32.const -5
      i32.const 5
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
      i32.const 3
      i32.and
      call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
      drop
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
      return
    end
    local.get 1
    call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
    drop
    call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
  )
  (@producers
    (processed-by "rustc" "1.90.0 (1159e78c4 2025-09-14)")
  )
  (@custom "target_features" (after code) "\08+\0bbulk-memory+\0fbulk-memory-opt+\16call-indirect-overlong+\0amultivalue+\0fmutable-globals+\13nontrapping-fptoint+\0freference-types+\08sign-ext")
)

"#;

pub(crate) const PREY: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "sense_at" (func $sense (param i32 i32) (result i32)))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (import "terrarium" "signal_broadcast" (func $broadcast (param i32) (result i32)))
  (import "terrarium" "random_byte" (func $rand (result i32)))
  (func $flee (param $dx i32) (param $dy i32) (param $away i32) (result i32)
    local.get $dx
    local.get $dy
    call $sense
    i32.const 2
    i32.ne
    if
      i32.const 0
      return
    end
    local.get $away
    call $move
    drop
    i32.const 1
    call $broadcast
    drop
    i32.const 1)
  (func (export "tick")
    i32.const 1
    i32.const 0
    i32.const 3
    call $flee
    if
      call $sleep
      return
    end
    i32.const -1
    i32.const 0
    i32.const 1
    call $flee
    if
      call $sleep
      return
    end
    i32.const 0
    i32.const 1
    i32.const 0
    call $flee
    if
      call $sleep
      return
    end
    i32.const 0
    i32.const -1
    i32.const 2
    call $flee
    if
      call $sleep
      return
    end
    call $rand
    i32.const 3
    i32.and
    call $move
    drop
    call $sleep)
)
"#;

pub(crate) const PREDATOR: &str = r#"
(module $strategy_predator.wasm
  (type (;0;) (func))
  (type (;1;) (func (result i32)))
  (type (;2;) (func (param i32) (result i32)))
  (type (;3;) (func (param i32 i32) (result i32)))
  (type (;4;) (func (param i32)))
  (import "terrarium" "sleep" (func $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E (;0;) (type 0)))
  (import "terrarium" "random_byte" (func $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E (;1;) (type 1)))
  (import "terrarium" "move" (func $_ZN15strategy_hunter4host4step17h54526a67a501102bE (;2;) (type 2)))
  (import "terrarium" "sense_at" (func $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E (;3;) (type 3)))
  (import "terrarium" "eat" (func $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE (;4;) (type 2)))
  (memory (;0;) 16)
  (global $__stack_pointer (;0;) (mut i32) i32.const 1048576)
  (global (;1;) i32 i32.const 1048576)
  (global (;2;) i32 i32.const 1048576)
  (export "memory" (memory 0))
  (export "tick" (func $tick))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2))
  (func $tick (;5;) (type 0)
    i32.const 2
    call $_ZN15strategy_hunter4tick17h13d0a9efadcd391bE
  )
  (func $_ZN15strategy_hunter4tick17h13d0a9efadcd391bE (;6;) (type 4) (param i32)
    (local i32)
    i32.const 0
    local.set 1
    block ;; label = @1
      block ;; label = @2
        i32.const 0
        i32.const -1
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.eq
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        i32.const 1
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.eq
        br_if 0 (;@2;)
        block ;; label = @3
          i32.const 0
          i32.const 1
          call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
          local.get 0
          i32.ne
          br_if 0 (;@3;)
          i32.const 2
          call $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE
          drop
          call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
          return
        end
        i32.const -1
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 1 (;@1;)
        i32.const 3
        local.set 1
      end
      local.get 1
      call $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE
      drop
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
      return
    end
    i32.const 1
    local.set 1
    block ;; label = @1
      i32.const 1
      i32.const 0
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      i32.const 0
      local.set 1
      i32.const 0
      i32.const -1
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -1
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 1
      local.set 1
      block ;; label = @2
        i32.const 0
        i32.const 1
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 2
        local.set 1
        br 1 (;@1;)
      end
      i32.const 1
      i32.const 1
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      i32.const 1
      local.set 1
      i32.const 1
      i32.const -1
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -1
        i32.const -1
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const -1
        i32.const 1
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      local.set 1
      block ;; label = @2
        i32.const 2
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      i32.const -2
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -2
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 2
      local.set 1
      i32.const 0
      i32.const 2
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const 2
        i32.const 2
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 2
        i32.const -2
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const -2
        i32.const -2
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 3
      local.set 1
      i32.const -2
      i32.const 2
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      i32.const 0
      local.set 1
      block ;; label = @2
        i32.const 3
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      i32.const -3
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -3
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 0
        i32.const 3
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 2
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 3
        i32.const 3
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 3
      local.set 1
      block ;; label = @2
        i32.const 3
        i32.const -3
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const -3
      i32.const -3
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -3
        i32.const 3
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      local.set 1
      block ;; label = @2
        i32.const 4
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      i32.const -4
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -4
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 0
        i32.const 4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 2
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 4
        i32.const 4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const 4
        i32.const -4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const -4
        i32.const -4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      block ;; label = @2
        i32.const -4
        i32.const 4
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      local.set 1
      block ;; label = @2
        i32.const 5
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 0
      i32.const -5
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -5
        i32.const 0
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 2
      local.set 1
      i32.const 0
      i32.const 5
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const 5
        i32.const 5
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 1
        local.set 1
        br 1 (;@1;)
      end
      i32.const 1
      local.set 1
      i32.const 5
      i32.const -5
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      block ;; label = @2
        i32.const -5
        i32.const -5
        call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
        local.get 0
        i32.ne
        br_if 0 (;@2;)
        i32.const 3
        local.set 1
        br 1 (;@1;)
      end
      i32.const 3
      local.set 1
      i32.const -5
      i32.const 5
      call $_ZN15strategy_hunter4host8sense_at17h3c882d122ee5b3d6E
      local.get 0
      i32.eq
      br_if 0 (;@1;)
      call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
      i32.const 3
      i32.and
      call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
      drop
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
      return
    end
    local.get 1
    call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
    drop
    call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
  )
  (@producers
    (processed-by "rustc" "1.90.0 (1159e78c4 2025-09-14)")
  )
  (@custom "target_features" (after code) "\08+\0bbulk-memory+\0fbulk-memory-opt+\16call-indirect-overlong+\0amultivalue+\0fmutable-globals+\13nontrapping-fptoint+\0freference-types+\08sign-ext")
)

"#;

const HAWK: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (import "terrarium" "recv" (func $recv (param i32) (result i32)))
  (import "terrarium" "pos_x" (func $pos_x (result i32)))
  (import "terrarium" "pos_y" (func $pos_y (result i32)))
  (import "terrarium" "random_byte" (func $rand (result i32)))
  (memory (export "memory") 1)
  (func $load (param $off i32) (result i32)
    local.get $off
    i32.load)
  (func $wander
    call $rand
    i32.const 3
    i32.and
    call $move
    drop
    call $sleep)
  (func $step_x (param $fx i32)
    call $pos_x
    local.get $fx
    i32.lt_s
    if
      i32.const 1
      call $move
      drop
      return
    end
    call $pos_x
    local.get $fx
    i32.gt_s
    if
      i32.const 3
      call $move
      drop
    end)
  (func $step_y (param $fy i32)
    call $pos_y
    local.get $fy
    i32.lt_s
    if
      i32.const 2
      call $move
      drop
      return
    end
    call $pos_y
    local.get $fy
    i32.gt_s
    if
      i32.const 0
      call $move
      drop
    end)
  (func (export "tick")
    (local $fx i32) (local $fy i32)
    i32.const 0
    call $recv
    i32.eqz
    if
      call $wander
      return
    end
    i32.const 12
    call $load
    i32.const 1
    i32.ne
    if
      call $wander
      return
    end
    i32.const 4
    call $load
    local.set $fx
    i32.const 8
    call $load
    local.set $fy
    local.get $fx
    call $step_x
    local.get $fy
    call $step_y
    call $sleep)
)
"#;
