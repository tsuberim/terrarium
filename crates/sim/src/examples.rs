//! Canonical WAT example programs (synced from strategies/ via build-strategies.sh).

pub struct ExampleProgram {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub code: &'static str,
}

pub const ALL: &[ExampleProgram] = &[
    ExampleProgram {
        id: "predator",
        name: "Predator",
        description: "Hunts creatures; broadcasts hunt ping (0x02) while chasing",
        code: PREDATOR,
    },
    ExampleProgram {
        id: "scavenger",
        name: "Scavenger",
        description: "Rushes prey alarms (0x01), eats corpses — competes with hawks",
        code: SCAVENGER,
    },
    ExampleProgram {
        id: "prey",
        name: "Prey",
        description: "Flees adjacent predators; alarm (0x01) draws hawks and scavengers",
        code: PREY,
    },
    ExampleProgram {
        id: "hawk",
        name: "Hawk",
        description: "Rushes prey alarms (0x01) to compete for the kill",
        code: HAWK,
    },
];

#[allow(dead_code)]
pub(crate) const IDLE: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (func (export "main")
    loop $l
      call $sleep
      br $l
    end)
)
"#;

#[allow(dead_code)]
pub(crate) const RUNNER: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (func (export "main")
    loop $l
      i32.const 0
      call $move
      drop
      call $sleep
      br $l
    end)
)
"#;

#[allow(dead_code)]
pub(crate) const BEACON: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (import "terrarium" "signal_broadcast" (func $broadcast (param i32) (result i32)))
  (func (export "main")
    loop $l
      i32.const 190
      call $broadcast
      drop
      i32.const 0
      call $move
      drop
      call $sleep
      br $l
    end)
)
"#;

#[allow(dead_code)]
pub(crate) const KAMIKAZE: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "energy" (func $energy (result i64)))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (func $main (export "main")
    loop $run
      call $energy
      i64.const 3000000
      i64.lt_s
      if (return) end
      i32.const 0
      call $move
      drop
      call $sleep
      br $run
    end)
)
"#;

#[allow(dead_code)]
const COLONIST: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (import "terrarium" "energy" (func $energy (result i64)))
  (import "terrarium" "move" (func $move (param i32) (result i32)))
  (import "terrarium" "random_byte" (func $rand (result i32)))
  (import "terrarium" "spawn" (func $spawn (param i32 i32) (result i32)))
  (func (export "main")
    call $energy
    i64.const 10000000
    i64.le_s
    if
      call $rand
      i32.const 6
      i32.rem_u
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
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32 i32 i32) (result i32)))
  (type (;2;) (func))
  (type (;3;) (func (result i32)))
  (type (;4;) (func (param i32) (result i32)))
  (type (;5;) (func (result i64)))
  (type (;6;) (func (param i32)))
  (type (;7;) (func (param i32 i32)))
  (type (;8;) (func (param i32 i32 i32)))
  (type (;9;) (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (type (;10;) (func (param i32 i32 i32 i32 i32) (result i32)))
  (type (;11;) (func (param i32 i32 i32 i32)))
  (import "terrarium" "sleep" (func $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E (;0;) (type 2)))
  (import "terrarium" "facing" (func $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E (;1;) (type 3)))
  (import "terrarium" "move" (func $_ZN15strategy_hunter4host4step17h54526a67a501102bE (;2;) (type 4)))
  (import "terrarium" "rotate" (func $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE (;3;) (type 4)))
  (import "terrarium" "random_byte" (func $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E (;4;) (type 3)))
  (import "terrarium" "energy" (func $_ZN15strategy_hunter4host6energy17h025e496adf3b9fedE (;5;) (type 5)))
  (import "terrarium" "sense" (func $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E (;6;) (type 1)))
  (import "terrarium" "spawn" (func $_ZN15strategy_hunter4host5spawn17h1c054dc162b77db4E (;7;) (type 0)))
  (import "terrarium" "eat" (func $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE (;8;) (type 4)))
  (import "terrarium" "recv" (func $_ZN15strategy_hunter4host4recv17h0668d301ceeeb97fE (;9;) (type 4)))
  (import "terrarium" "pos_x" (func $_ZN15strategy_hunter4host5pos_x17h83d553ce0faa30ccE (;10;) (type 3)))
  (import "terrarium" "pos_y" (func $_ZN15strategy_hunter4host5pos_y17h042d3565584392bdE (;11;) (type 3)))
  (table (;0;) 2 2 funcref)
  (memory (;0;) 17)
  (global $__stack_pointer (;0;) (mut i32) i32.const 1048576)
  (global (;1;) i32 i32.const 1048988)
  (global (;2;) i32 i32.const 1048992)
  (export "memory" (memory 0))
  (export "main" (func $main))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2))
  (elem (;0;) (i32.const 1) func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hacc3ad28f4de0a4dE)
  (func $main (;12;) (type 2)
    loop ;; label = @1
      call $_ZN15strategy_hunter14scavenger_tick17haaa34e6990b5ca87E
      br 0 (;@1;)
    end
  )
  (func $_RNvCsj4CZ6flxxfE_7___rustc17rust_begin_unwind (;13;) (type 6) (param i32)
    call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
    loop ;; label = @1
      br 0 (;@1;)
    end
  )
  (func $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE (;14;) (type 7) (param i32 i32)
    (local i32)
    i32.const 0
    local.set 2
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            local.get 0
            i32.const 1
            i32.add
            br_table 1 (;@3;) 0 (;@4;) 2 (;@2;) 3 (;@1;)
          end
          i32.const 2
          i32.const 5
          i32.const 0
          local.get 1
          i32.const 1
          i32.eq
          select
          local.get 1
          i32.const -1
          i32.eq
          select
          local.set 2
          br 2 (;@1;)
        end
        local.get 1
        i32.const 1
        i32.eq
        i32.const 2
        i32.shl
        i32.const 3
        local.get 1
        select
        local.set 2
        br 1 (;@1;)
      end
      local.get 1
      i32.const -1
      i32.eq
      local.set 2
    end
    block ;; label = @1
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 1
      i32.const 6
      i32.add
      local.get 1
      local.get 1
      i32.const 0
      i32.lt_s
      select
      br_if 0 (;@1;)
      i32.const 0
      call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
      drop
      return
    end
    block ;; label = @1
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 2
      i32.const 6
      i32.add
      local.get 2
      local.get 2
      i32.const 0
      i32.lt_s
      select
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.const -6
      i32.add
      local.get 2
      i32.const 4
      i32.lt_u
      select
      call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
      drop
    end
  )
  (func $_ZN15strategy_hunter11maybe_clone17h2c98f5d73388d033E (;15;) (type 3) (result i32)
    (local i32 i32 i32)
    i32.const 0
    local.set 0
    block ;; label = @1
      call $_ZN15strategy_hunter4host6energy17h025e496adf3b9fedE
      i64.const 5000001
      i64.lt_s
      br_if 0 (;@1;)
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
            i32.const 6
            i32.rem_s
            local.tee 1
            i32.const -1
            i32.le_s
            br_if 0 (;@4;)
            local.get 1
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 1 (;@3;)
            i32.const 1
            i32.const -5
            local.get 1
            i32.const 5
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 2
            i32.const -4
            local.get 1
            i32.const 4
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 3
            i32.const -3
            local.get 1
            i32.const 3
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 4
            i32.const -2
            local.get 1
            i32.const 2
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            local.set 0
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            local.get 1
            i32.const -1
            i32.add
            i32.const 5
            local.get 1
            select
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 1
            i32.const 1048576
            i32.add
            i32.load
            local.get 1
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            br_if 3 (;@1;)
            br 2 (;@2;)
          end
          local.get 1
          i32.const 6
          i32.const 1048644
          call $_ZN4core9panicking18panic_bounds_check17hf4028f296c44236fE
          unreachable
        end
        local.get 1
        local.set 2
      end
      block ;; label = @2
        block ;; label = @3
          call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
          local.get 2
          i32.ne
          br_if 0 (;@3;)
          i32.const 0
          i32.const 2000000
          call $_ZN15strategy_hunter4host5spawn17h1c054dc162b77db4E
          drop
          br 1 (;@2;)
        end
        local.get 2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        i32.sub
        i32.const 6
        i32.rem_s
        local.tee 1
        i32.const 6
        i32.add
        local.get 1
        local.get 1
        i32.const 0
        i32.lt_s
        select
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 1
        i32.const -6
        i32.add
        local.get 1
        i32.const 4
        i32.lt_u
        select
        call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
        drop
      end
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
      i32.const 1
      local.set 0
    end
    local.get 0
  )
  (func $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E (;16;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i64 i32 i32 i32 i64 i32 i32)
    i32.const 0
    local.set 2
    i32.const 1
    i32.const 0
    i32.const 1048928
    call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
    drop
    block ;; label = @1
      block ;; label = @2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 1
        local.set 2
        i32.const 1
        i32.const -1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        i32.const -1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 2
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const -1
        i32.const 0
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 3
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const -1
        i32.const 1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 4
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        i32.const 1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 5
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        block ;; label = @3
          block ;; label = @4
            local.get 0
            br_if 0 (;@4;)
            i64.const 0
            local.set 4
            i32.const 0
            local.set 5
            i32.const 0
            local.set 6
            i32.const 0
            local.set 7
            i32.const 1
            local.set 0
            loop ;; label = @5
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 0
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 0
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const -1
                i32.add
                local.tee 2
                i32.add
                br_if 0 (;@6;)
              end
              local.get 0
              local.set 3
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 3
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 2
                i32.const 1
                i32.add
                local.set 2
                local.get 0
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                i32.add
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const 1
                i32.add
                local.tee 2
                i32.ne
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.add
                local.tee 9
                local.get 0
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 0
                  local.set 6
                  local.get 9
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const 1
                i32.add
                local.tee 2
                i32.ne
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              local.get 0
              local.set 3
              loop ;; label = @6
                local.get 2
                local.get 3
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 3
                  local.set 6
                  local.get 2
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 2
                i32.const 1
                i32.add
                local.set 2
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                br_if 0 (;@6;)
              end
              local.get 0
              i32.const 1
              i32.add
              local.tee 0
              i32.const 6
              i32.ne
              br_if 0 (;@5;)
              br 2 (;@3;)
            end
          end
          i64.const 0
          local.set 4
          i32.const 0
          local.set 5
          i32.const 0
          local.set 6
          i32.const 0
          local.set 7
          i32.const 1
          local.set 0
          loop ;; label = @4
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 0
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 3
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 3
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 0
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const -1
              i32.add
              local.tee 2
              i32.add
              br_if 0 (;@5;)
            end
            local.get 0
            local.set 3
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 3
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 2
              i32.const 1
              i32.add
              local.set 2
              local.get 0
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              i32.add
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const 1
              i32.add
              local.tee 2
              i32.ne
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.add
              local.tee 9
              local.get 0
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 10
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 10
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 0
                local.set 6
                local.get 9
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const 1
              i32.add
              local.tee 2
              i32.ne
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            local.get 0
            local.set 3
            loop ;; label = @5
              local.get 2
              local.get 3
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 3
                local.set 6
                local.get 2
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 2
              i32.const 1
              i32.add
              local.set 2
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              br_if 0 (;@5;)
            end
            local.get 0
            i32.const 1
            i32.add
            local.tee 0
            i32.const 6
            i32.ne
            br_if 0 (;@4;)
          end
        end
        block ;; label = @3
          block ;; label = @4
            local.get 7
            i32.const 1
            i32.and
            br_if 0 (;@4;)
            i32.const 0
            local.set 0
            local.get 1
            i32.eqz
            br_if 3 (;@1;)
            call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
            local.set 2
            call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
            local.get 2
            i32.const 6
            i32.rem_s
            local.tee 2
            i32.ne
            br_if 1 (;@3;)
            i32.const 0
            call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
            drop
            i32.const 1
            return
          end
          local.get 5
          local.get 6
          call $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE
          i32.const 1
          return
        end
        i32.const 1
        local.set 0
        local.get 2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        i32.sub
        i32.const 6
        i32.rem_s
        local.tee 2
        i32.const 6
        i32.add
        local.get 2
        local.get 2
        i32.const 0
        i32.lt_s
        select
        local.tee 2
        i32.eqz
        br_if 1 (;@1;)
        local.get 2
        local.get 2
        i32.const -6
        i32.add
        local.get 2
        i32.const 4
        i32.lt_u
        select
        call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
        drop
        br 1 (;@1;)
      end
      block ;; label = @2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        local.get 2
        i32.ne
        br_if 0 (;@2;)
        i32.const 0
        call $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE
        drop
        i32.const 1
        return
      end
      i32.const 1
      local.set 0
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 2
      i32.const 6
      i32.add
      local.get 2
      local.get 2
      i32.const 0
      i32.lt_s
      select
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.const -6
      i32.add
      local.get 2
      i32.const 4
      i32.lt_u
      select
      call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
      drop
      i32.const 1
      return
    end
    local.get 0
  )
  (func $_ZN15strategy_hunter14scavenger_tick17haaa34e6990b5ca87E (;17;) (type 2)
    (local i32)
    block ;; label = @1
      call $_ZN15strategy_hunter11maybe_clone17h2c98f5d73388d033E
      br_if 0 (;@1;)
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              i32.const 1048952
              call $_ZN15strategy_hunter4host4recv17h0668d301ceeeb97fE
              i32.eqz
              br_if 0 (;@5;)
              i32.const 0
              i32.load offset=1048964 align=1
              i32.const 1
              i32.eq
              br_if 1 (;@4;)
            end
            i32.const 1
            i32.const 1
            call $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E
            br_if 1 (;@3;)
            br 2 (;@2;)
          end
          i32.const 0
          i32.load offset=1048960 align=1
          local.set 0
          i32.const 0
          i32.load offset=1048956 align=1
          call $_ZN15strategy_hunter4host5pos_x17h83d553ce0faa30ccE
          i32.sub
          local.get 0
          call $_ZN15strategy_hunter4host5pos_y17h042d3565584392bdE
          i32.sub
          call $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE
        end
        block ;; label = @3
          i32.const 1048952
          call $_ZN15strategy_hunter4host4recv17h0668d301ceeeb97fE
          i32.eqz
          br_if 0 (;@3;)
          i32.const 0
          i32.load offset=1048964 align=1
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          i32.const 0
          i32.load offset=1048960 align=1
          local.set 0
          i32.const 0
          i32.load offset=1048956 align=1
          call $_ZN15strategy_hunter4host5pos_x17h83d553ce0faa30ccE
          i32.sub
          local.get 0
          call $_ZN15strategy_hunter4host5pos_y17h042d3565584392bdE
          i32.sub
          call $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE
          br 1 (;@2;)
        end
        i32.const 1
        i32.const 1
        call $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E
        drop
      end
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
    end
  )
  (func $_ZN4core9panicking18panic_bounds_check17hf4028f296c44236fE (;18;) (type 8) (param i32 i32 i32)
    (local i32 i64)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1048712
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    i32.const 1
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.tee 4
    local.get 3
    i64.extend_i32_u
    i64.or
    i64.store offset=40
    local.get 3
    local.get 4
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call $_ZN4core9panicking9panic_fmt17h808dbde205a89691E
    unreachable
  )
  (func $_ZN4core9panicking9panic_fmt17h808dbde205a89691E (;19;) (type 7) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 1
    i32.store16 offset=12
    local.get 2
    local.get 1
    i32.store offset=8
    local.get 2
    local.get 0
    i32.store offset=4
    local.get 2
    i32.const 4
    i32.add
    call $_RNvCsj4CZ6flxxfE_7___rustc17rust_begin_unwind
    unreachable
  )
  (func $_ZN4core3str5count14do_count_chars17haa2c4f188ad8cef2E (;20;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    block ;; label = @1
      block ;; label = @2
        local.get 1
        local.get 0
        i32.const 3
        i32.add
        i32.const -4
        i32.and
        local.tee 2
        local.get 0
        i32.sub
        local.tee 3
        i32.lt_u
        br_if 0 (;@2;)
        local.get 1
        local.get 3
        i32.sub
        local.tee 4
        i32.const 4
        i32.lt_u
        br_if 0 (;@2;)
        local.get 4
        i32.const 3
        i32.and
        local.set 5
        i32.const 0
        local.set 6
        i32.const 0
        local.set 1
        block ;; label = @3
          local.get 2
          local.get 0
          i32.eq
          local.tee 7
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          block ;; label = @4
            block ;; label = @5
              local.get 0
              local.get 2
              i32.sub
              local.tee 8
              i32.const -4
              i32.le_u
              br_if 0 (;@5;)
              i32.const 0
              local.set 9
              br 1 (;@4;)
            end
            i32.const 0
            local.set 9
            loop ;; label = @5
              local.get 1
              local.get 0
              local.get 9
              i32.add
              local.tee 2
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 1
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 2
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 3
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.set 1
              local.get 9
              i32.const 4
              i32.add
              local.tee 9
              br_if 0 (;@5;)
            end
          end
          local.get 7
          br_if 0 (;@3;)
          local.get 0
          local.get 9
          i32.add
          local.set 2
          loop ;; label = @4
            local.get 1
            local.get 2
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 2
            i32.const 1
            i32.add
            local.set 2
            local.get 8
            i32.const 1
            i32.add
            local.tee 8
            br_if 0 (;@4;)
          end
        end
        local.get 0
        local.get 3
        i32.add
        local.set 0
        block ;; label = @3
          local.get 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 4
          i32.const -4
          i32.and
          i32.add
          local.tee 2
          i32.load8_s
          i32.const -65
          i32.gt_s
          local.set 6
          local.get 5
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=1
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
          local.get 5
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=2
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
        end
        local.get 4
        i32.const 2
        i32.shr_u
        local.set 8
        local.get 6
        local.get 1
        i32.add
        local.set 3
        loop ;; label = @3
          local.get 0
          local.set 4
          local.get 8
          i32.eqz
          br_if 2 (;@1;)
          local.get 8
          i32.const 192
          local.get 8
          i32.const 192
          i32.lt_u
          select
          local.tee 6
          i32.const 3
          i32.and
          local.set 7
          local.get 6
          i32.const 2
          i32.shl
          local.set 5
          i32.const 0
          local.set 2
          block ;; label = @4
            local.get 8
            i32.const 4
            i32.lt_u
            br_if 0 (;@4;)
            local.get 4
            local.get 5
            i32.const 1008
            i32.and
            i32.add
            local.set 9
            i32.const 0
            local.set 2
            local.get 4
            local.set 1
            loop ;; label = @5
              local.get 1
              i32.const 12
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.const 8
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.const 4
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 2
              i32.add
              i32.add
              i32.add
              i32.add
              local.set 2
              local.get 1
              i32.const 16
              i32.add
              local.tee 1
              local.get 9
              i32.ne
              br_if 0 (;@5;)
            end
          end
          local.get 8
          local.get 6
          i32.sub
          local.set 8
          local.get 4
          local.get 5
          i32.add
          local.set 0
          local.get 2
          i32.const 8
          i32.shr_u
          i32.const 16711935
          i32.and
          local.get 2
          i32.const 16711935
          i32.and
          i32.add
          i32.const 65537
          i32.mul
          i32.const 16
          i32.shr_u
          local.get 3
          i32.add
          local.set 3
          local.get 7
          i32.eqz
          br_if 0 (;@3;)
        end
        local.get 4
        local.get 6
        i32.const 252
        i32.and
        i32.const 2
        i32.shl
        i32.add
        local.tee 2
        i32.load
        local.tee 1
        i32.const -1
        i32.xor
        i32.const 7
        i32.shr_u
        local.get 1
        i32.const 6
        i32.shr_u
        i32.or
        i32.const 16843009
        i32.and
        local.set 1
        block ;; label = @3
          local.get 7
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=4
          local.tee 0
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 0
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
          local.get 7
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=8
          local.tee 2
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 2
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
        end
        local.get 1
        i32.const 8
        i32.shr_u
        i32.const 459007
        i32.and
        local.get 1
        i32.const 16711935
        i32.and
        i32.add
        i32.const 65537
        i32.mul
        i32.const 16
        i32.shr_u
        local.get 3
        i32.add
        return
      end
      block ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        i32.const 0
        return
      end
      local.get 1
      i32.const 3
      i32.and
      local.set 9
      block ;; label = @2
        block ;; label = @3
          local.get 1
          i32.const 4
          i32.ge_u
          br_if 0 (;@3;)
          i32.const 0
          local.set 3
          i32.const 0
          local.set 2
          br 1 (;@2;)
        end
        local.get 1
        i32.const -4
        i32.and
        local.set 8
        i32.const 0
        local.set 3
        i32.const 0
        local.set 2
        loop ;; label = @3
          local.get 3
          local.get 0
          local.get 2
          i32.add
          local.tee 1
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 1
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 2
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 3
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 3
          local.get 8
          local.get 2
          i32.const 4
          i32.add
          local.tee 2
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 9
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      i32.add
      local.set 1
      loop ;; label = @2
        local.get 3
        local.get 1
        i32.load8_s
        i32.const -65
        i32.gt_s
        i32.add
        local.set 3
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 9
        i32.const -1
        i32.add
        local.tee 9
        br_if 0 (;@2;)
      end
    end
    local.get 3
  )
  (func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hacc3ad28f4de0a4dE (;21;) (type 0) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 8
    i32.add
    local.get 0
    i32.load
    local.get 2
    i32.const 22
    i32.add
    i32.const 10
    call $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb4e91fd13b3ed913E
    local.get 1
    i32.const 1
    i32.const 1
    i32.const 0
    local.get 2
    i32.load offset=8
    local.get 2
    i32.load offset=12
    call $_ZN4core3fmt9Formatter12pad_integral17hc6b3558773c79a2cE
    local.set 0
    local.get 2
    i32.const 32
    i32.add
    global.set $__stack_pointer
    local.get 0
  )
  (func $_ZN4core3fmt9Formatter12pad_integral17hc6b3558773c79a2cE (;22;) (type 9) (param i32 i32 i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i64)
    block ;; label = @1
      block ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        local.get 5
        i32.const 1
        i32.add
        local.set 6
        local.get 0
        i32.load offset=8
        local.set 7
        i32.const 45
        local.set 8
        br 1 (;@1;)
      end
      i32.const 43
      i32.const 1114112
      local.get 0
      i32.load offset=8
      local.tee 7
      i32.const 2097152
      i32.and
      local.tee 1
      select
      local.set 8
      local.get 1
      i32.const 21
      i32.shr_u
      local.get 5
      i32.add
      local.set 6
    end
    block ;; label = @1
      block ;; label = @2
        local.get 7
        i32.const 8388608
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        local.set 2
        br 1 (;@1;)
      end
      block ;; label = @2
        block ;; label = @3
          local.get 3
          i32.const 16
          i32.lt_u
          br_if 0 (;@3;)
          local.get 2
          local.get 3
          call $_ZN4core3str5count14do_count_chars17haa2c4f188ad8cef2E
          local.set 1
          br 1 (;@2;)
        end
        block ;; label = @3
          local.get 3
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          br 1 (;@2;)
        end
        local.get 3
        i32.const 3
        i32.and
        local.set 9
        block ;; label = @3
          block ;; label = @4
            local.get 3
            i32.const 4
            i32.ge_u
            br_if 0 (;@4;)
            i32.const 0
            local.set 1
            i32.const 0
            local.set 10
            br 1 (;@3;)
          end
          local.get 3
          i32.const 12
          i32.and
          local.set 11
          i32.const 0
          local.set 1
          i32.const 0
          local.set 10
          loop ;; label = @4
            local.get 1
            local.get 2
            local.get 10
            i32.add
            local.tee 12
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 1
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 2
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 3
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 11
            local.get 10
            i32.const 4
            i32.add
            local.tee 10
            i32.ne
            br_if 0 (;@4;)
          end
        end
        local.get 9
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 10
        i32.add
        local.set 12
        loop ;; label = @3
          local.get 1
          local.get 12
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 1
          local.get 12
          i32.const 1
          i32.add
          local.set 12
          local.get 9
          i32.const -1
          i32.add
          local.tee 9
          br_if 0 (;@3;)
        end
      end
      local.get 1
      local.get 6
      i32.add
      local.set 6
    end
    block ;; label = @1
      block ;; label = @2
        local.get 6
        local.get 0
        i32.load16_u offset=12
        local.tee 11
        i32.ge_u
        br_if 0 (;@2;)
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              local.get 7
              i32.const 16777216
              i32.and
              br_if 0 (;@5;)
              local.get 11
              local.get 6
              i32.sub
              local.set 13
              i32.const 0
              local.set 1
              i32.const 0
              local.set 11
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    local.get 7
                    i32.const 29
                    i32.shr_u
                    i32.const 3
                    i32.and
                    br_table 2 (;@6;) 0 (;@8;) 1 (;@7;) 0 (;@8;) 2 (;@6;)
                  end
                  local.get 13
                  local.set 11
                  br 1 (;@6;)
                end
                local.get 13
                i32.const 65534
                i32.and
                i32.const 1
                i32.shr_u
                local.set 11
              end
              local.get 7
              i32.const 2097151
              i32.and
              local.set 6
              local.get 0
              i32.load offset=4
              local.set 9
              local.get 0
              i32.load
              local.set 10
              loop ;; label = @6
                local.get 1
                i32.const 65535
                i32.and
                local.get 11
                i32.const 65535
                i32.and
                i32.ge_u
                br_if 2 (;@4;)
                i32.const 1
                local.set 12
                local.get 1
                i32.const 1
                i32.add
                local.set 1
                local.get 10
                local.get 6
                local.get 9
                i32.load offset=16
                call_indirect (type 0)
                i32.eqz
                br_if 0 (;@6;)
                br 5 (;@1;)
              end
            end
            local.get 0
            local.get 0
            i64.load offset=8 align=4
            local.tee 14
            i32.wrap_i64
            i32.const -1612709888
            i32.and
            i32.const 536870960
            i32.or
            i32.store offset=8
            i32.const 1
            local.set 12
            local.get 0
            i32.load
            local.tee 10
            local.get 0
            i32.load offset=4
            local.tee 9
            local.get 8
            local.get 2
            local.get 3
            call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
            br_if 3 (;@1;)
            i32.const 0
            local.set 1
            local.get 11
            local.get 6
            i32.sub
            i32.const 65535
            i32.and
            local.set 2
            loop ;; label = @5
              local.get 1
              i32.const 65535
              i32.and
              local.get 2
              i32.ge_u
              br_if 2 (;@3;)
              i32.const 1
              local.set 12
              local.get 1
              i32.const 1
              i32.add
              local.set 1
              local.get 10
              i32.const 48
              local.get 9
              i32.load offset=16
              call_indirect (type 0)
              i32.eqz
              br_if 0 (;@5;)
              br 4 (;@1;)
            end
          end
          i32.const 1
          local.set 12
          local.get 10
          local.get 9
          local.get 8
          local.get 2
          local.get 3
          call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
          br_if 2 (;@1;)
          local.get 10
          local.get 4
          local.get 5
          local.get 9
          i32.load offset=12
          call_indirect (type 1)
          br_if 2 (;@1;)
          i32.const 0
          local.set 1
          local.get 13
          local.get 11
          i32.sub
          i32.const 65535
          i32.and
          local.set 0
          loop ;; label = @4
            local.get 1
            i32.const 65535
            i32.and
            local.tee 2
            local.get 0
            i32.lt_u
            local.set 12
            local.get 2
            local.get 0
            i32.ge_u
            br_if 3 (;@1;)
            local.get 1
            i32.const 1
            i32.add
            local.set 1
            local.get 10
            local.get 6
            local.get 9
            i32.load offset=16
            call_indirect (type 0)
            i32.eqz
            br_if 0 (;@4;)
            br 3 (;@1;)
          end
        end
        i32.const 1
        local.set 12
        local.get 10
        local.get 4
        local.get 5
        local.get 9
        i32.load offset=12
        call_indirect (type 1)
        br_if 1 (;@1;)
        local.get 0
        local.get 14
        i64.store offset=8 align=4
        i32.const 0
        return
      end
      i32.const 1
      local.set 12
      local.get 0
      i32.load
      local.tee 1
      local.get 0
      i32.load offset=4
      local.tee 10
      local.get 8
      local.get 2
      local.get 3
      call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
      br_if 0 (;@1;)
      local.get 1
      local.get 4
      local.get 5
      local.get 10
      i32.load offset=12
      call_indirect (type 1)
      local.set 12
    end
    local.get 12
  )
  (func $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE (;23;) (type 10) (param i32 i32 i32 i32 i32) (result i32)
    block ;; label = @1
      local.get 2
      i32.const 1114112
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      local.get 1
      i32.load offset=16
      call_indirect (type 0)
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    block ;; label = @1
      local.get 3
      br_if 0 (;@1;)
      i32.const 0
      return
    end
    local.get 0
    local.get 3
    local.get 4
    local.get 1
    i32.load offset=12
    call_indirect (type 1)
  )
  (func $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb4e91fd13b3ed913E (;24;) (type 11) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    local.get 1
    local.set 4
    local.get 3
    local.set 5
    block ;; label = @1
      local.get 1
      i32.const 1000
      i32.lt_u
      br_if 0 (;@1;)
      local.get 2
      i32.const -4
      i32.add
      local.set 6
      local.get 3
      local.set 5
      local.get 1
      local.set 7
      loop ;; label = @2
        local.get 6
        local.get 5
        i32.add
        local.tee 8
        i32.const 1
        i32.add
        local.get 7
        local.get 7
        i32.const 10000
        i32.div_u
        local.tee 4
        i32.const 10000
        i32.mul
        i32.sub
        local.tee 9
        i32.const 65535
        i32.and
        i32.const 100
        i32.div_u
        local.tee 10
        i32.const 1
        i32.shl
        local.tee 11
        i32.const 1048729
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        local.get 11
        i32.const 1048728
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        i32.const 3
        i32.add
        local.get 9
        local.get 10
        i32.const 100
        i32.mul
        i32.sub
        i32.const 65535
        i32.and
        i32.const 1
        i32.shl
        local.tee 9
        i32.const 1048729
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        i32.const 2
        i32.add
        local.get 9
        i32.const 1048728
        i32.add
        i32.load8_u
        i32.store8
        local.get 5
        i32.const -4
        i32.add
        local.set 5
        local.get 7
        i32.const 9999999
        i32.gt_u
        local.set 8
        local.get 4
        local.set 7
        local.get 8
        br_if 0 (;@2;)
      end
    end
    block ;; label = @1
      block ;; label = @2
        local.get 4
        i32.const 9
        i32.gt_u
        br_if 0 (;@2;)
        local.get 4
        local.set 7
        br 1 (;@1;)
      end
      local.get 2
      local.get 5
      i32.add
      i32.const -1
      i32.add
      local.get 4
      local.get 4
      i32.const 65535
      i32.and
      i32.const 100
      i32.div_u
      local.tee 7
      i32.const 100
      i32.mul
      i32.sub
      i32.const 65535
      i32.and
      i32.const 1
      i32.shl
      local.tee 8
      i32.const 1048729
      i32.add
      i32.load8_u
      i32.store8
      local.get 2
      local.get 5
      i32.const -2
      i32.add
      local.tee 5
      i32.add
      local.get 8
      i32.const 1048728
      i32.add
      i32.load8_u
      i32.store8
    end
    block ;; label = @1
      block ;; label = @2
        local.get 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 7
        i32.eqz
        br_if 1 (;@1;)
      end
      local.get 2
      local.get 5
      i32.const -1
      i32.add
      local.tee 5
      i32.add
      local.get 7
      i32.const 1
      i32.shl
      i32.const 30
      i32.and
      i32.const 1048729
      i32.add
      i32.load8_u
      i32.store8
    end
    local.get 0
    local.get 3
    local.get 5
    i32.sub
    i32.store offset=4
    local.get 0
    local.get 2
    local.get 5
    i32.add
    i32.store
  )
  (data $.rodata (;0;) (i32.const 1048576) "\01\00\00\00\00\00\00\00\01\00\00\00\ff\ff\ff\ff\00\00\00\00\ff\ff\ff\ff\ff\ff\ff\ff\00\00\00\00\ff\ff\ff\ff\01\00\00\00\00\00\00\00\01\00\00\00hunter/src/lib.rs\00\00\000\00\10\00\11\00\00\00\cf\00\00\00\1c\00\00\00index out of bounds: the len is  but the index is \00\00T\00\10\00 \00\00\00t\00\10\00\12\00\00\0000010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899")
  (@producers
    (language "Rust" "")
    (processed-by "rustc" "1.90.0 (1159e78c4 2025-09-14)")
  )
  (@custom "target_features" (after data) "\08+\0bbulk-memory+\0fbulk-memory-opt+\16call-indirect-overlong+\0amultivalue+\0fmutable-globals+\13nontrapping-fptoint+\0freference-types+\08sign-ext")
)

"#;

pub(crate) const PREY: &str = r#"
(module $strategy_prey.wasm
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32 i32 i32) (result i32)))
  (type (;2;) (func))
  (type (;3;) (func (result i32)))
  (type (;4;) (func (param i32) (result i32)))
  (type (;5;) (func (result i64)))
  (type (;6;) (func (param i32)))
  (type (;7;) (func (param i32 i32)))
  (type (;8;) (func (param i32 i32 i32)))
  (type (;9;) (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (type (;10;) (func (param i32 i32 i32 i32 i32) (result i32)))
  (type (;11;) (func (param i32 i32 i32 i32)))
  (import "terrarium" "sleep" (func $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E (;0;) (type 2)))
  (import "terrarium" "facing" (func $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E (;1;) (type 3)))
  (import "terrarium" "move" (func $_ZN15strategy_hunter4host4step17h54526a67a501102bE (;2;) (type 4)))
  (import "terrarium" "rotate" (func $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE (;3;) (type 4)))
  (import "terrarium" "random_byte" (func $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E (;4;) (type 3)))
  (import "terrarium" "energy" (func $_ZN15strategy_hunter4host6energy17h025e496adf3b9fedE (;5;) (type 5)))
  (import "terrarium" "sense" (func $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E (;6;) (type 1)))
  (import "terrarium" "spawn" (func $_ZN15strategy_hunter4host5spawn17h1c054dc162b77db4E (;7;) (type 0)))
  (import "terrarium" "eat" (func $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE (;8;) (type 4)))
  (import "terrarium" "signal_broadcast" (func $_ZN15strategy_hunter4host16signal_broadcast17h575c2111c62c1639E (;9;) (type 4)))
  (table (;0;) 2 2 funcref)
  (memory (;0;) 17)
  (global $__stack_pointer (;0;) (mut i32) i32.const 1048576)
  (global (;1;) i32 i32.const 1048952)
  (global (;2;) i32 i32.const 1048960)
  (export "memory" (memory 0))
  (export "main" (func $main))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2))
  (elem (;0;) (i32.const 1) func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hacc3ad28f4de0a4dE)
  (func $main (;10;) (type 2)
    loop ;; label = @1
      call $_ZN15strategy_hunter9prey_tick17h74a7446dc3f74c04E
      br 0 (;@1;)
    end
  )
  (func $_RNvCsj4CZ6flxxfE_7___rustc17rust_begin_unwind (;11;) (type 6) (param i32)
    call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
    loop ;; label = @1
      br 0 (;@1;)
    end
  )
  (func $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE (;12;) (type 7) (param i32 i32)
    (local i32)
    i32.const 0
    local.set 2
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            local.get 0
            i32.const 1
            i32.add
            br_table 1 (;@3;) 0 (;@4;) 2 (;@2;) 3 (;@1;)
          end
          i32.const 2
          i32.const 5
          i32.const 0
          local.get 1
          i32.const 1
          i32.eq
          select
          local.get 1
          i32.const -1
          i32.eq
          select
          local.set 2
          br 2 (;@1;)
        end
        local.get 1
        i32.const 1
        i32.eq
        i32.const 2
        i32.shl
        i32.const 3
        local.get 1
        select
        local.set 2
        br 1 (;@1;)
      end
      local.get 1
      i32.const -1
      i32.eq
      local.set 2
    end
    block ;; label = @1
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 1
      i32.const 6
      i32.add
      local.get 1
      local.get 1
      i32.const 0
      i32.lt_s
      select
      br_if 0 (;@1;)
      i32.const 0
      call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
      drop
      return
    end
    block ;; label = @1
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 2
      i32.const 6
      i32.add
      local.get 2
      local.get 2
      i32.const 0
      i32.lt_s
      select
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.const -6
      i32.add
      local.get 2
      i32.const 4
      i32.lt_u
      select
      call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
      drop
    end
  )
  (func $_ZN15strategy_hunter11maybe_clone17h2c98f5d73388d033E (;13;) (type 3) (result i32)
    (local i32 i32 i32)
    i32.const 0
    local.set 0
    block ;; label = @1
      call $_ZN15strategy_hunter4host6energy17h025e496adf3b9fedE
      i64.const 5000001
      i64.lt_s
      br_if 0 (;@1;)
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
            i32.const 6
            i32.rem_s
            local.tee 1
            i32.const -1
            i32.le_s
            br_if 0 (;@4;)
            local.get 1
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 1 (;@3;)
            i32.const 1
            i32.const -5
            local.get 1
            i32.const 5
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 2
            i32.const -4
            local.get 1
            i32.const 4
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 3
            i32.const -3
            local.get 1
            i32.const 3
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 4
            i32.const -2
            local.get 1
            i32.const 2
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            local.set 0
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            local.get 1
            i32.const -1
            i32.add
            i32.const 5
            local.get 1
            select
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 1
            i32.const 1048576
            i32.add
            i32.load
            local.get 1
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            br_if 3 (;@1;)
            br 2 (;@2;)
          end
          local.get 1
          i32.const 6
          i32.const 1048644
          call $_ZN4core9panicking18panic_bounds_check17hf4028f296c44236fE
          unreachable
        end
        local.get 1
        local.set 2
      end
      block ;; label = @2
        block ;; label = @3
          call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
          local.get 2
          i32.ne
          br_if 0 (;@3;)
          i32.const 0
          i32.const 2000000
          call $_ZN15strategy_hunter4host5spawn17h1c054dc162b77db4E
          drop
          br 1 (;@2;)
        end
        local.get 2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        i32.sub
        i32.const 6
        i32.rem_s
        local.tee 1
        i32.const 6
        i32.add
        local.get 1
        local.get 1
        i32.const 0
        i32.lt_s
        select
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 1
        i32.const -6
        i32.add
        local.get 1
        i32.const 4
        i32.lt_u
        select
        call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
        drop
      end
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
      i32.const 1
      local.set 0
    end
    local.get 0
  )
  (func $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E (;14;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i64 i32 i32 i32 i64 i32 i32)
    i32.const 0
    local.set 2
    i32.const 1
    i32.const 0
    i32.const 1048928
    call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
    drop
    block ;; label = @1
      block ;; label = @2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 1
        local.set 2
        i32.const 1
        i32.const -1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        i32.const -1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 2
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const -1
        i32.const 0
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 3
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const -1
        i32.const 1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 4
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        i32.const 1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 5
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        block ;; label = @3
          block ;; label = @4
            local.get 0
            br_if 0 (;@4;)
            i64.const 0
            local.set 4
            i32.const 0
            local.set 5
            i32.const 0
            local.set 6
            i32.const 0
            local.set 7
            i32.const 1
            local.set 0
            loop ;; label = @5
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 0
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 0
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const -1
                i32.add
                local.tee 2
                i32.add
                br_if 0 (;@6;)
              end
              local.get 0
              local.set 3
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 3
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 2
                i32.const 1
                i32.add
                local.set 2
                local.get 0
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                i32.add
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const 1
                i32.add
                local.tee 2
                i32.ne
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.add
                local.tee 9
                local.get 0
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 0
                  local.set 6
                  local.get 9
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const 1
                i32.add
                local.tee 2
                i32.ne
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              local.get 0
              local.set 3
              loop ;; label = @6
                local.get 2
                local.get 3
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 3
                  local.set 6
                  local.get 2
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 2
                i32.const 1
                i32.add
                local.set 2
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                br_if 0 (;@6;)
              end
              local.get 0
              i32.const 1
              i32.add
              local.tee 0
              i32.const 6
              i32.ne
              br_if 0 (;@5;)
              br 2 (;@3;)
            end
          end
          i64.const 0
          local.set 4
          i32.const 0
          local.set 5
          i32.const 0
          local.set 6
          i32.const 0
          local.set 7
          i32.const 1
          local.set 0
          loop ;; label = @4
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 0
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 3
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 3
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 0
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const -1
              i32.add
              local.tee 2
              i32.add
              br_if 0 (;@5;)
            end
            local.get 0
            local.set 3
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 3
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 2
              i32.const 1
              i32.add
              local.set 2
              local.get 0
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              i32.add
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const 1
              i32.add
              local.tee 2
              i32.ne
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.add
              local.tee 9
              local.get 0
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 10
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 10
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 0
                local.set 6
                local.get 9
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const 1
              i32.add
              local.tee 2
              i32.ne
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            local.get 0
            local.set 3
            loop ;; label = @5
              local.get 2
              local.get 3
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 3
                local.set 6
                local.get 2
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 2
              i32.const 1
              i32.add
              local.set 2
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              br_if 0 (;@5;)
            end
            local.get 0
            i32.const 1
            i32.add
            local.tee 0
            i32.const 6
            i32.ne
            br_if 0 (;@4;)
          end
        end
        block ;; label = @3
          block ;; label = @4
            local.get 7
            i32.const 1
            i32.and
            br_if 0 (;@4;)
            i32.const 0
            local.set 0
            local.get 1
            i32.eqz
            br_if 3 (;@1;)
            call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
            local.set 2
            call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
            local.get 2
            i32.const 6
            i32.rem_s
            local.tee 2
            i32.ne
            br_if 1 (;@3;)
            i32.const 0
            call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
            drop
            i32.const 1
            return
          end
          local.get 5
          local.get 6
          call $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE
          i32.const 1
          return
        end
        i32.const 1
        local.set 0
        local.get 2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        i32.sub
        i32.const 6
        i32.rem_s
        local.tee 2
        i32.const 6
        i32.add
        local.get 2
        local.get 2
        i32.const 0
        i32.lt_s
        select
        local.tee 2
        i32.eqz
        br_if 1 (;@1;)
        local.get 2
        local.get 2
        i32.const -6
        i32.add
        local.get 2
        i32.const 4
        i32.lt_u
        select
        call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
        drop
        br 1 (;@1;)
      end
      block ;; label = @2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        local.get 2
        i32.ne
        br_if 0 (;@2;)
        i32.const 0
        call $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE
        drop
        i32.const 1
        return
      end
      i32.const 1
      local.set 0
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 2
      i32.const 6
      i32.add
      local.get 2
      local.get 2
      i32.const 0
      i32.lt_s
      select
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.const -6
      i32.add
      local.get 2
      i32.const 4
      i32.lt_u
      select
      call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
      drop
      i32.const 1
      return
    end
    local.get 0
  )
  (func $_ZN15strategy_hunter9prey_tick17h74a7446dc3f74c04E (;15;) (type 2)
    (local i32 i32)
    block ;; label = @1
      call $_ZN15strategy_hunter11maybe_clone17h2c98f5d73388d033E
      br_if 0 (;@1;)
      i32.const 1
      i32.const 0
      i32.const 1048928
      call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
      drop
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              block ;; label = @6
                i32.const 0
                i32.load offset=1048928
                local.tee 0
                i32.const 2
                i32.eq
                br_if 0 (;@6;)
                i32.const 1
                i32.const -1
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                i32.const 0
                i32.load offset=1048928
                i32.const 2
                i32.eq
                br_if 0 (;@6;)
                i32.const 0
                i32.const -1
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                i32.const 2
                local.set 0
                i32.const 0
                i32.load offset=1048928
                i32.const 2
                i32.eq
                br_if 1 (;@5;)
                i32.const -1
                i32.const 0
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                i32.const -3
                local.set 1
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 2
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 3
                  local.set 0
                  br 3 (;@4;)
                end
                i32.const -1
                i32.const 1
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 2
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 4
                  local.set 0
                  br 3 (;@4;)
                end
                i32.const 0
                i32.const 1
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                i32.const 0
                i32.load offset=1048928
                i32.const 2
                i32.ne
                br_if 3 (;@3;)
                i32.const 5
                local.set 0
                br 2 (;@4;)
              end
              local.get 0
              i32.const 2
              i32.ne
              local.set 0
            end
            i32.const 3
            local.set 1
          end
          block ;; label = @4
            call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
            local.get 1
            local.get 0
            i32.add
            local.tee 0
            i32.ne
            br_if 0 (;@4;)
            i32.const 0
            call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
            drop
            i32.const 1
            call $_ZN15strategy_hunter4host16signal_broadcast17h575c2111c62c1639E
            drop
            br 2 (;@2;)
          end
          local.get 0
          call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
          i32.sub
          i32.const 6
          i32.rem_s
          local.tee 0
          i32.const 6
          i32.add
          local.get 0
          local.get 0
          i32.const 0
          i32.lt_s
          select
          local.tee 0
          i32.eqz
          br_if 1 (;@2;)
          local.get 0
          local.get 0
          i32.const -6
          i32.add
          local.get 0
          i32.const 4
          i32.lt_u
          select
          call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
          drop
          br 1 (;@2;)
        end
        i32.const 0
        i32.const 0
        call $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E
        br_if 0 (;@2;)
        call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
        local.set 0
        block ;; label = @3
          call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
          local.get 0
          i32.const 6
          i32.rem_s
          local.tee 0
          i32.ne
          br_if 0 (;@3;)
          i32.const 0
          call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
          drop
          br 1 (;@2;)
        end
        local.get 0
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        i32.sub
        i32.const 6
        i32.rem_s
        local.tee 0
        i32.const 6
        i32.add
        local.get 0
        local.get 0
        i32.const 0
        i32.lt_s
        select
        local.tee 0
        i32.eqz
        br_if 0 (;@2;)
        local.get 0
        local.get 0
        i32.const -6
        i32.add
        local.get 0
        i32.const 4
        i32.lt_u
        select
        call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
        drop
      end
      i32.const 1
      i32.const 0
      i32.const 1048928
      call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
      drop
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              i32.const 0
              i32.load offset=1048928
              local.tee 0
              i32.const 2
              i32.eq
              br_if 0 (;@5;)
              i32.const 1
              i32.const -1
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i32.load offset=1048928
              i32.const 2
              i32.eq
              br_if 0 (;@5;)
              i32.const 0
              i32.const -1
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 2
              local.set 0
              i32.const 0
              i32.load offset=1048928
              i32.const 2
              i32.eq
              br_if 1 (;@4;)
              i32.const -1
              i32.const 0
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const -3
              local.set 1
              block ;; label = @6
                i32.const 0
                i32.load offset=1048928
                i32.const 2
                i32.ne
                br_if 0 (;@6;)
                i32.const 3
                local.set 0
                br 3 (;@3;)
              end
              i32.const -1
              i32.const 1
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              block ;; label = @6
                i32.const 0
                i32.load offset=1048928
                i32.const 2
                i32.ne
                br_if 0 (;@6;)
                i32.const 4
                local.set 0
                br 3 (;@3;)
              end
              i32.const 0
              i32.const 1
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              block ;; label = @6
                i32.const 0
                i32.load offset=1048928
                i32.const 2
                i32.ne
                br_if 0 (;@6;)
                i32.const 5
                local.set 0
                br 3 (;@3;)
              end
              i32.const 0
              i32.const 0
              call $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E
              br_if 3 (;@2;)
              call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
              local.set 0
              block ;; label = @6
                call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
                local.get 0
                i32.const 6
                i32.rem_s
                local.tee 0
                i32.eq
                br_if 0 (;@6;)
                local.get 0
                call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
                i32.sub
                i32.const 6
                i32.rem_s
                local.tee 0
                i32.const 6
                i32.add
                local.get 0
                local.get 0
                i32.const 0
                i32.lt_s
                select
                local.tee 0
                i32.eqz
                br_if 4 (;@2;)
                local.get 0
                local.get 0
                i32.const -6
                i32.add
                local.get 0
                i32.const 4
                i32.lt_u
                select
                call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
                drop
                br 4 (;@2;)
              end
              i32.const 0
              call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
              drop
              br 3 (;@2;)
            end
            local.get 0
            i32.const 2
            i32.ne
            local.set 0
          end
          i32.const 3
          local.set 1
        end
        block ;; label = @3
          call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
          local.get 1
          local.get 0
          i32.add
          local.tee 0
          i32.eq
          br_if 0 (;@3;)
          local.get 0
          call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
          i32.sub
          i32.const 6
          i32.rem_s
          local.tee 0
          i32.const 6
          i32.add
          local.get 0
          local.get 0
          i32.const 0
          i32.lt_s
          select
          local.tee 0
          i32.eqz
          br_if 1 (;@2;)
          local.get 0
          local.get 0
          i32.const -6
          i32.add
          local.get 0
          i32.const 4
          i32.lt_u
          select
          call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
          drop
          br 1 (;@2;)
        end
        i32.const 0
        call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
        drop
        i32.const 1
        call $_ZN15strategy_hunter4host16signal_broadcast17h575c2111c62c1639E
        drop
      end
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
    end
  )
  (func $_ZN4core9panicking18panic_bounds_check17hf4028f296c44236fE (;16;) (type 8) (param i32 i32 i32)
    (local i32 i64)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1048712
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    i32.const 1
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.tee 4
    local.get 3
    i64.extend_i32_u
    i64.or
    i64.store offset=40
    local.get 3
    local.get 4
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call $_ZN4core9panicking9panic_fmt17h808dbde205a89691E
    unreachable
  )
  (func $_ZN4core9panicking9panic_fmt17h808dbde205a89691E (;17;) (type 7) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 1
    i32.store16 offset=12
    local.get 2
    local.get 1
    i32.store offset=8
    local.get 2
    local.get 0
    i32.store offset=4
    local.get 2
    i32.const 4
    i32.add
    call $_RNvCsj4CZ6flxxfE_7___rustc17rust_begin_unwind
    unreachable
  )
  (func $_ZN4core3str5count14do_count_chars17haa2c4f188ad8cef2E (;18;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    block ;; label = @1
      block ;; label = @2
        local.get 1
        local.get 0
        i32.const 3
        i32.add
        i32.const -4
        i32.and
        local.tee 2
        local.get 0
        i32.sub
        local.tee 3
        i32.lt_u
        br_if 0 (;@2;)
        local.get 1
        local.get 3
        i32.sub
        local.tee 4
        i32.const 4
        i32.lt_u
        br_if 0 (;@2;)
        local.get 4
        i32.const 3
        i32.and
        local.set 5
        i32.const 0
        local.set 6
        i32.const 0
        local.set 1
        block ;; label = @3
          local.get 2
          local.get 0
          i32.eq
          local.tee 7
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          block ;; label = @4
            block ;; label = @5
              local.get 0
              local.get 2
              i32.sub
              local.tee 8
              i32.const -4
              i32.le_u
              br_if 0 (;@5;)
              i32.const 0
              local.set 9
              br 1 (;@4;)
            end
            i32.const 0
            local.set 9
            loop ;; label = @5
              local.get 1
              local.get 0
              local.get 9
              i32.add
              local.tee 2
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 1
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 2
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 3
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.set 1
              local.get 9
              i32.const 4
              i32.add
              local.tee 9
              br_if 0 (;@5;)
            end
          end
          local.get 7
          br_if 0 (;@3;)
          local.get 0
          local.get 9
          i32.add
          local.set 2
          loop ;; label = @4
            local.get 1
            local.get 2
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 2
            i32.const 1
            i32.add
            local.set 2
            local.get 8
            i32.const 1
            i32.add
            local.tee 8
            br_if 0 (;@4;)
          end
        end
        local.get 0
        local.get 3
        i32.add
        local.set 0
        block ;; label = @3
          local.get 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 4
          i32.const -4
          i32.and
          i32.add
          local.tee 2
          i32.load8_s
          i32.const -65
          i32.gt_s
          local.set 6
          local.get 5
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=1
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
          local.get 5
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=2
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
        end
        local.get 4
        i32.const 2
        i32.shr_u
        local.set 8
        local.get 6
        local.get 1
        i32.add
        local.set 3
        loop ;; label = @3
          local.get 0
          local.set 4
          local.get 8
          i32.eqz
          br_if 2 (;@1;)
          local.get 8
          i32.const 192
          local.get 8
          i32.const 192
          i32.lt_u
          select
          local.tee 6
          i32.const 3
          i32.and
          local.set 7
          local.get 6
          i32.const 2
          i32.shl
          local.set 5
          i32.const 0
          local.set 2
          block ;; label = @4
            local.get 8
            i32.const 4
            i32.lt_u
            br_if 0 (;@4;)
            local.get 4
            local.get 5
            i32.const 1008
            i32.and
            i32.add
            local.set 9
            i32.const 0
            local.set 2
            local.get 4
            local.set 1
            loop ;; label = @5
              local.get 1
              i32.const 12
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.const 8
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.const 4
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 2
              i32.add
              i32.add
              i32.add
              i32.add
              local.set 2
              local.get 1
              i32.const 16
              i32.add
              local.tee 1
              local.get 9
              i32.ne
              br_if 0 (;@5;)
            end
          end
          local.get 8
          local.get 6
          i32.sub
          local.set 8
          local.get 4
          local.get 5
          i32.add
          local.set 0
          local.get 2
          i32.const 8
          i32.shr_u
          i32.const 16711935
          i32.and
          local.get 2
          i32.const 16711935
          i32.and
          i32.add
          i32.const 65537
          i32.mul
          i32.const 16
          i32.shr_u
          local.get 3
          i32.add
          local.set 3
          local.get 7
          i32.eqz
          br_if 0 (;@3;)
        end
        local.get 4
        local.get 6
        i32.const 252
        i32.and
        i32.const 2
        i32.shl
        i32.add
        local.tee 2
        i32.load
        local.tee 1
        i32.const -1
        i32.xor
        i32.const 7
        i32.shr_u
        local.get 1
        i32.const 6
        i32.shr_u
        i32.or
        i32.const 16843009
        i32.and
        local.set 1
        block ;; label = @3
          local.get 7
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=4
          local.tee 0
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 0
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
          local.get 7
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=8
          local.tee 2
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 2
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
        end
        local.get 1
        i32.const 8
        i32.shr_u
        i32.const 459007
        i32.and
        local.get 1
        i32.const 16711935
        i32.and
        i32.add
        i32.const 65537
        i32.mul
        i32.const 16
        i32.shr_u
        local.get 3
        i32.add
        return
      end
      block ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        i32.const 0
        return
      end
      local.get 1
      i32.const 3
      i32.and
      local.set 9
      block ;; label = @2
        block ;; label = @3
          local.get 1
          i32.const 4
          i32.ge_u
          br_if 0 (;@3;)
          i32.const 0
          local.set 3
          i32.const 0
          local.set 2
          br 1 (;@2;)
        end
        local.get 1
        i32.const -4
        i32.and
        local.set 8
        i32.const 0
        local.set 3
        i32.const 0
        local.set 2
        loop ;; label = @3
          local.get 3
          local.get 0
          local.get 2
          i32.add
          local.tee 1
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 1
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 2
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 3
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 3
          local.get 8
          local.get 2
          i32.const 4
          i32.add
          local.tee 2
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 9
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      i32.add
      local.set 1
      loop ;; label = @2
        local.get 3
        local.get 1
        i32.load8_s
        i32.const -65
        i32.gt_s
        i32.add
        local.set 3
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 9
        i32.const -1
        i32.add
        local.tee 9
        br_if 0 (;@2;)
      end
    end
    local.get 3
  )
  (func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hacc3ad28f4de0a4dE (;19;) (type 0) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 8
    i32.add
    local.get 0
    i32.load
    local.get 2
    i32.const 22
    i32.add
    i32.const 10
    call $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb4e91fd13b3ed913E
    local.get 1
    i32.const 1
    i32.const 1
    i32.const 0
    local.get 2
    i32.load offset=8
    local.get 2
    i32.load offset=12
    call $_ZN4core3fmt9Formatter12pad_integral17hc6b3558773c79a2cE
    local.set 0
    local.get 2
    i32.const 32
    i32.add
    global.set $__stack_pointer
    local.get 0
  )
  (func $_ZN4core3fmt9Formatter12pad_integral17hc6b3558773c79a2cE (;20;) (type 9) (param i32 i32 i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i64)
    block ;; label = @1
      block ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        local.get 5
        i32.const 1
        i32.add
        local.set 6
        local.get 0
        i32.load offset=8
        local.set 7
        i32.const 45
        local.set 8
        br 1 (;@1;)
      end
      i32.const 43
      i32.const 1114112
      local.get 0
      i32.load offset=8
      local.tee 7
      i32.const 2097152
      i32.and
      local.tee 1
      select
      local.set 8
      local.get 1
      i32.const 21
      i32.shr_u
      local.get 5
      i32.add
      local.set 6
    end
    block ;; label = @1
      block ;; label = @2
        local.get 7
        i32.const 8388608
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        local.set 2
        br 1 (;@1;)
      end
      block ;; label = @2
        block ;; label = @3
          local.get 3
          i32.const 16
          i32.lt_u
          br_if 0 (;@3;)
          local.get 2
          local.get 3
          call $_ZN4core3str5count14do_count_chars17haa2c4f188ad8cef2E
          local.set 1
          br 1 (;@2;)
        end
        block ;; label = @3
          local.get 3
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          br 1 (;@2;)
        end
        local.get 3
        i32.const 3
        i32.and
        local.set 9
        block ;; label = @3
          block ;; label = @4
            local.get 3
            i32.const 4
            i32.ge_u
            br_if 0 (;@4;)
            i32.const 0
            local.set 1
            i32.const 0
            local.set 10
            br 1 (;@3;)
          end
          local.get 3
          i32.const 12
          i32.and
          local.set 11
          i32.const 0
          local.set 1
          i32.const 0
          local.set 10
          loop ;; label = @4
            local.get 1
            local.get 2
            local.get 10
            i32.add
            local.tee 12
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 1
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 2
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 3
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 11
            local.get 10
            i32.const 4
            i32.add
            local.tee 10
            i32.ne
            br_if 0 (;@4;)
          end
        end
        local.get 9
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 10
        i32.add
        local.set 12
        loop ;; label = @3
          local.get 1
          local.get 12
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 1
          local.get 12
          i32.const 1
          i32.add
          local.set 12
          local.get 9
          i32.const -1
          i32.add
          local.tee 9
          br_if 0 (;@3;)
        end
      end
      local.get 1
      local.get 6
      i32.add
      local.set 6
    end
    block ;; label = @1
      block ;; label = @2
        local.get 6
        local.get 0
        i32.load16_u offset=12
        local.tee 11
        i32.ge_u
        br_if 0 (;@2;)
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              local.get 7
              i32.const 16777216
              i32.and
              br_if 0 (;@5;)
              local.get 11
              local.get 6
              i32.sub
              local.set 13
              i32.const 0
              local.set 1
              i32.const 0
              local.set 11
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    local.get 7
                    i32.const 29
                    i32.shr_u
                    i32.const 3
                    i32.and
                    br_table 2 (;@6;) 0 (;@8;) 1 (;@7;) 0 (;@8;) 2 (;@6;)
                  end
                  local.get 13
                  local.set 11
                  br 1 (;@6;)
                end
                local.get 13
                i32.const 65534
                i32.and
                i32.const 1
                i32.shr_u
                local.set 11
              end
              local.get 7
              i32.const 2097151
              i32.and
              local.set 6
              local.get 0
              i32.load offset=4
              local.set 9
              local.get 0
              i32.load
              local.set 10
              loop ;; label = @6
                local.get 1
                i32.const 65535
                i32.and
                local.get 11
                i32.const 65535
                i32.and
                i32.ge_u
                br_if 2 (;@4;)
                i32.const 1
                local.set 12
                local.get 1
                i32.const 1
                i32.add
                local.set 1
                local.get 10
                local.get 6
                local.get 9
                i32.load offset=16
                call_indirect (type 0)
                i32.eqz
                br_if 0 (;@6;)
                br 5 (;@1;)
              end
            end
            local.get 0
            local.get 0
            i64.load offset=8 align=4
            local.tee 14
            i32.wrap_i64
            i32.const -1612709888
            i32.and
            i32.const 536870960
            i32.or
            i32.store offset=8
            i32.const 1
            local.set 12
            local.get 0
            i32.load
            local.tee 10
            local.get 0
            i32.load offset=4
            local.tee 9
            local.get 8
            local.get 2
            local.get 3
            call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
            br_if 3 (;@1;)
            i32.const 0
            local.set 1
            local.get 11
            local.get 6
            i32.sub
            i32.const 65535
            i32.and
            local.set 2
            loop ;; label = @5
              local.get 1
              i32.const 65535
              i32.and
              local.get 2
              i32.ge_u
              br_if 2 (;@3;)
              i32.const 1
              local.set 12
              local.get 1
              i32.const 1
              i32.add
              local.set 1
              local.get 10
              i32.const 48
              local.get 9
              i32.load offset=16
              call_indirect (type 0)
              i32.eqz
              br_if 0 (;@5;)
              br 4 (;@1;)
            end
          end
          i32.const 1
          local.set 12
          local.get 10
          local.get 9
          local.get 8
          local.get 2
          local.get 3
          call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
          br_if 2 (;@1;)
          local.get 10
          local.get 4
          local.get 5
          local.get 9
          i32.load offset=12
          call_indirect (type 1)
          br_if 2 (;@1;)
          i32.const 0
          local.set 1
          local.get 13
          local.get 11
          i32.sub
          i32.const 65535
          i32.and
          local.set 0
          loop ;; label = @4
            local.get 1
            i32.const 65535
            i32.and
            local.tee 2
            local.get 0
            i32.lt_u
            local.set 12
            local.get 2
            local.get 0
            i32.ge_u
            br_if 3 (;@1;)
            local.get 1
            i32.const 1
            i32.add
            local.set 1
            local.get 10
            local.get 6
            local.get 9
            i32.load offset=16
            call_indirect (type 0)
            i32.eqz
            br_if 0 (;@4;)
            br 3 (;@1;)
          end
        end
        i32.const 1
        local.set 12
        local.get 10
        local.get 4
        local.get 5
        local.get 9
        i32.load offset=12
        call_indirect (type 1)
        br_if 1 (;@1;)
        local.get 0
        local.get 14
        i64.store offset=8 align=4
        i32.const 0
        return
      end
      i32.const 1
      local.set 12
      local.get 0
      i32.load
      local.tee 1
      local.get 0
      i32.load offset=4
      local.tee 10
      local.get 8
      local.get 2
      local.get 3
      call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
      br_if 0 (;@1;)
      local.get 1
      local.get 4
      local.get 5
      local.get 10
      i32.load offset=12
      call_indirect (type 1)
      local.set 12
    end
    local.get 12
  )
  (func $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE (;21;) (type 10) (param i32 i32 i32 i32 i32) (result i32)
    block ;; label = @1
      local.get 2
      i32.const 1114112
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      local.get 1
      i32.load offset=16
      call_indirect (type 0)
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    block ;; label = @1
      local.get 3
      br_if 0 (;@1;)
      i32.const 0
      return
    end
    local.get 0
    local.get 3
    local.get 4
    local.get 1
    i32.load offset=12
    call_indirect (type 1)
  )
  (func $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb4e91fd13b3ed913E (;22;) (type 11) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    local.get 1
    local.set 4
    local.get 3
    local.set 5
    block ;; label = @1
      local.get 1
      i32.const 1000
      i32.lt_u
      br_if 0 (;@1;)
      local.get 2
      i32.const -4
      i32.add
      local.set 6
      local.get 3
      local.set 5
      local.get 1
      local.set 7
      loop ;; label = @2
        local.get 6
        local.get 5
        i32.add
        local.tee 8
        i32.const 1
        i32.add
        local.get 7
        local.get 7
        i32.const 10000
        i32.div_u
        local.tee 4
        i32.const 10000
        i32.mul
        i32.sub
        local.tee 9
        i32.const 65535
        i32.and
        i32.const 100
        i32.div_u
        local.tee 10
        i32.const 1
        i32.shl
        local.tee 11
        i32.const 1048729
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        local.get 11
        i32.const 1048728
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        i32.const 3
        i32.add
        local.get 9
        local.get 10
        i32.const 100
        i32.mul
        i32.sub
        i32.const 65535
        i32.and
        i32.const 1
        i32.shl
        local.tee 9
        i32.const 1048729
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        i32.const 2
        i32.add
        local.get 9
        i32.const 1048728
        i32.add
        i32.load8_u
        i32.store8
        local.get 5
        i32.const -4
        i32.add
        local.set 5
        local.get 7
        i32.const 9999999
        i32.gt_u
        local.set 8
        local.get 4
        local.set 7
        local.get 8
        br_if 0 (;@2;)
      end
    end
    block ;; label = @1
      block ;; label = @2
        local.get 4
        i32.const 9
        i32.gt_u
        br_if 0 (;@2;)
        local.get 4
        local.set 7
        br 1 (;@1;)
      end
      local.get 2
      local.get 5
      i32.add
      i32.const -1
      i32.add
      local.get 4
      local.get 4
      i32.const 65535
      i32.and
      i32.const 100
      i32.div_u
      local.tee 7
      i32.const 100
      i32.mul
      i32.sub
      i32.const 65535
      i32.and
      i32.const 1
      i32.shl
      local.tee 8
      i32.const 1048729
      i32.add
      i32.load8_u
      i32.store8
      local.get 2
      local.get 5
      i32.const -2
      i32.add
      local.tee 5
      i32.add
      local.get 8
      i32.const 1048728
      i32.add
      i32.load8_u
      i32.store8
    end
    block ;; label = @1
      block ;; label = @2
        local.get 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 7
        i32.eqz
        br_if 1 (;@1;)
      end
      local.get 2
      local.get 5
      i32.const -1
      i32.add
      local.tee 5
      i32.add
      local.get 7
      i32.const 1
      i32.shl
      i32.const 30
      i32.and
      i32.const 1048729
      i32.add
      i32.load8_u
      i32.store8
    end
    local.get 0
    local.get 3
    local.get 5
    i32.sub
    i32.store offset=4
    local.get 0
    local.get 2
    local.get 5
    i32.add
    i32.store
  )
  (data $.rodata (;0;) (i32.const 1048576) "\01\00\00\00\00\00\00\00\01\00\00\00\ff\ff\ff\ff\00\00\00\00\ff\ff\ff\ff\ff\ff\ff\ff\00\00\00\00\ff\ff\ff\ff\01\00\00\00\00\00\00\00\01\00\00\00hunter/src/lib.rs\00\00\000\00\10\00\11\00\00\00\cf\00\00\00\1c\00\00\00index out of bounds: the len is  but the index is \00\00T\00\10\00 \00\00\00t\00\10\00\12\00\00\0000010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899")
  (@producers
    (language "Rust" "")
    (processed-by "rustc" "1.90.0 (1159e78c4 2025-09-14)")
  )
  (@custom "target_features" (after data) "\08+\0bbulk-memory+\0fbulk-memory-opt+\16call-indirect-overlong+\0amultivalue+\0fmutable-globals+\13nontrapping-fptoint+\0freference-types+\08sign-ext")
)

"#;

pub(crate) const PREDATOR: &str = r#"
(module $strategy_predator.wasm
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32 i32 i32) (result i32)))
  (type (;2;) (func))
  (type (;3;) (func (result i32)))
  (type (;4;) (func (param i32) (result i32)))
  (type (;5;) (func (result i64)))
  (type (;6;) (func (param i32)))
  (type (;7;) (func (param i32 i32)))
  (type (;8;) (func (param i32 i32 i32)))
  (type (;9;) (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (type (;10;) (func (param i32 i32 i32 i32 i32) (result i32)))
  (type (;11;) (func (param i32 i32 i32 i32)))
  (import "terrarium" "sleep" (func $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E (;0;) (type 2)))
  (import "terrarium" "facing" (func $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E (;1;) (type 3)))
  (import "terrarium" "move" (func $_ZN15strategy_hunter4host4step17h54526a67a501102bE (;2;) (type 4)))
  (import "terrarium" "rotate" (func $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE (;3;) (type 4)))
  (import "terrarium" "random_byte" (func $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E (;4;) (type 3)))
  (import "terrarium" "energy" (func $_ZN15strategy_hunter4host6energy17h025e496adf3b9fedE (;5;) (type 5)))
  (import "terrarium" "sense" (func $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E (;6;) (type 1)))
  (import "terrarium" "spawn" (func $_ZN15strategy_hunter4host5spawn17h1c054dc162b77db4E (;7;) (type 0)))
  (import "terrarium" "eat" (func $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE (;8;) (type 4)))
  (import "terrarium" "signal_broadcast" (func $_ZN15strategy_hunter4host16signal_broadcast17h575c2111c62c1639E (;9;) (type 4)))
  (import "terrarium" "hit" (func $_ZN15strategy_hunter4host3hit17h862a6806f604c65cE (;10;) (type 4)))
  (table (;0;) 2 2 funcref)
  (memory (;0;) 17)
  (global $__stack_pointer (;0;) (mut i32) i32.const 1048576)
  (global (;1;) i32 i32.const 1048952)
  (global (;2;) i32 i32.const 1048960)
  (export "memory" (memory 0))
  (export "main" (func $main))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2))
  (elem (;0;) (i32.const 1) func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hacc3ad28f4de0a4dE)
  (func $main (;11;) (type 2)
    loop ;; label = @1
      call $_ZN15strategy_hunter13predator_tick17hdd4a25299cdb0f63E
      br 0 (;@1;)
    end
  )
  (func $_RNvCsj4CZ6flxxfE_7___rustc17rust_begin_unwind (;12;) (type 6) (param i32)
    call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
    loop ;; label = @1
      br 0 (;@1;)
    end
  )
  (func $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE (;13;) (type 7) (param i32 i32)
    (local i32)
    i32.const 0
    local.set 2
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            local.get 0
            i32.const 1
            i32.add
            br_table 1 (;@3;) 0 (;@4;) 2 (;@2;) 3 (;@1;)
          end
          i32.const 2
          i32.const 5
          i32.const 0
          local.get 1
          i32.const 1
          i32.eq
          select
          local.get 1
          i32.const -1
          i32.eq
          select
          local.set 2
          br 2 (;@1;)
        end
        local.get 1
        i32.const 1
        i32.eq
        i32.const 2
        i32.shl
        i32.const 3
        local.get 1
        select
        local.set 2
        br 1 (;@1;)
      end
      local.get 1
      i32.const -1
      i32.eq
      local.set 2
    end
    block ;; label = @1
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 1
      i32.const 6
      i32.add
      local.get 1
      local.get 1
      i32.const 0
      i32.lt_s
      select
      br_if 0 (;@1;)
      i32.const 0
      call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
      drop
      return
    end
    block ;; label = @1
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 2
      i32.const 6
      i32.add
      local.get 2
      local.get 2
      i32.const 0
      i32.lt_s
      select
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.const -6
      i32.add
      local.get 2
      i32.const 4
      i32.lt_u
      select
      call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
      drop
    end
  )
  (func $_ZN15strategy_hunter11wander_step17ha908672c1f1e31deE (;14;) (type 2)
    (local i32)
    call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
    local.set 0
    block ;; label = @1
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      local.get 0
      i32.const 6
      i32.rem_s
      local.tee 0
      i32.ne
      br_if 0 (;@1;)
      i32.const 0
      call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
      drop
      return
    end
    block ;; label = @1
      local.get 0
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 0
      i32.const 6
      i32.add
      local.get 0
      local.get 0
      i32.const 0
      i32.lt_s
      select
      local.tee 0
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 0
      i32.const -6
      i32.add
      local.get 0
      i32.const 4
      i32.lt_u
      select
      call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
      drop
    end
  )
  (func $_ZN15strategy_hunter11maybe_clone17h2c98f5d73388d033E (;15;) (type 3) (result i32)
    (local i32 i32 i32)
    i32.const 0
    local.set 0
    block ;; label = @1
      call $_ZN15strategy_hunter4host6energy17h025e496adf3b9fedE
      i64.const 5000001
      i64.lt_s
      br_if 0 (;@1;)
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
            i32.const 6
            i32.rem_s
            local.tee 1
            i32.const -1
            i32.le_s
            br_if 0 (;@4;)
            local.get 1
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 1 (;@3;)
            i32.const 1
            i32.const -5
            local.get 1
            i32.const 5
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 2
            i32.const -4
            local.get 1
            i32.const 4
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 3
            i32.const -3
            local.get 1
            i32.const 3
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 4
            i32.const -2
            local.get 1
            i32.const 2
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            local.set 0
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            local.get 1
            i32.const -1
            i32.add
            i32.const 5
            local.get 1
            select
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 1
            i32.const 1048576
            i32.add
            i32.load
            local.get 1
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            br_if 3 (;@1;)
            br 2 (;@2;)
          end
          local.get 1
          i32.const 6
          i32.const 1048644
          call $_ZN4core9panicking18panic_bounds_check17hf4028f296c44236fE
          unreachable
        end
        local.get 1
        local.set 2
      end
      block ;; label = @2
        block ;; label = @3
          call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
          local.get 2
          i32.ne
          br_if 0 (;@3;)
          i32.const 0
          i32.const 2000000
          call $_ZN15strategy_hunter4host5spawn17h1c054dc162b77db4E
          drop
          br 1 (;@2;)
        end
        local.get 2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        i32.sub
        i32.const 6
        i32.rem_s
        local.tee 1
        i32.const 6
        i32.add
        local.get 1
        local.get 1
        i32.const 0
        i32.lt_s
        select
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 1
        i32.const -6
        i32.add
        local.get 1
        i32.const 4
        i32.lt_u
        select
        call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
        drop
      end
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
      i32.const 1
      local.set 0
    end
    local.get 0
  )
  (func $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E (;16;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i64 i32 i32 i32 i64 i32 i32)
    i32.const 0
    local.set 2
    i32.const 1
    i32.const 0
    i32.const 1048928
    call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
    drop
    block ;; label = @1
      block ;; label = @2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 1
        local.set 2
        i32.const 1
        i32.const -1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        i32.const -1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 2
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const -1
        i32.const 0
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 3
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const -1
        i32.const 1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 4
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        i32.const 1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 5
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        block ;; label = @3
          block ;; label = @4
            local.get 0
            br_if 0 (;@4;)
            i64.const 0
            local.set 4
            i32.const 0
            local.set 5
            i32.const 0
            local.set 6
            i32.const 0
            local.set 7
            i32.const 1
            local.set 0
            loop ;; label = @5
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 0
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 0
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const -1
                i32.add
                local.tee 2
                i32.add
                br_if 0 (;@6;)
              end
              local.get 0
              local.set 3
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 3
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 2
                i32.const 1
                i32.add
                local.set 2
                local.get 0
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                i32.add
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const 1
                i32.add
                local.tee 2
                i32.ne
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.add
                local.tee 9
                local.get 0
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 0
                  local.set 6
                  local.get 9
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const 1
                i32.add
                local.tee 2
                i32.ne
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              local.get 0
              local.set 3
              loop ;; label = @6
                local.get 2
                local.get 3
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 3
                  local.set 6
                  local.get 2
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 2
                i32.const 1
                i32.add
                local.set 2
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                br_if 0 (;@6;)
              end
              local.get 0
              i32.const 1
              i32.add
              local.tee 0
              i32.const 6
              i32.ne
              br_if 0 (;@5;)
              br 2 (;@3;)
            end
          end
          i64.const 0
          local.set 4
          i32.const 0
          local.set 5
          i32.const 0
          local.set 6
          i32.const 0
          local.set 7
          i32.const 1
          local.set 0
          loop ;; label = @4
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 0
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 3
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 3
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 0
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const -1
              i32.add
              local.tee 2
              i32.add
              br_if 0 (;@5;)
            end
            local.get 0
            local.set 3
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 3
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 2
              i32.const 1
              i32.add
              local.set 2
              local.get 0
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              i32.add
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const 1
              i32.add
              local.tee 2
              i32.ne
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.add
              local.tee 9
              local.get 0
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 10
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 10
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 0
                local.set 6
                local.get 9
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const 1
              i32.add
              local.tee 2
              i32.ne
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            local.get 0
            local.set 3
            loop ;; label = @5
              local.get 2
              local.get 3
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 3
                local.set 6
                local.get 2
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 2
              i32.const 1
              i32.add
              local.set 2
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              br_if 0 (;@5;)
            end
            local.get 0
            i32.const 1
            i32.add
            local.tee 0
            i32.const 6
            i32.ne
            br_if 0 (;@4;)
          end
        end
        block ;; label = @3
          block ;; label = @4
            local.get 7
            i32.const 1
            i32.and
            br_if 0 (;@4;)
            i32.const 0
            local.set 0
            local.get 1
            i32.eqz
            br_if 3 (;@1;)
            call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
            local.set 2
            call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
            local.get 2
            i32.const 6
            i32.rem_s
            local.tee 2
            i32.ne
            br_if 1 (;@3;)
            i32.const 0
            call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
            drop
            i32.const 1
            return
          end
          local.get 5
          local.get 6
          call $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE
          i32.const 1
          return
        end
        i32.const 1
        local.set 0
        local.get 2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        i32.sub
        i32.const 6
        i32.rem_s
        local.tee 2
        i32.const 6
        i32.add
        local.get 2
        local.get 2
        i32.const 0
        i32.lt_s
        select
        local.tee 2
        i32.eqz
        br_if 1 (;@1;)
        local.get 2
        local.get 2
        i32.const -6
        i32.add
        local.get 2
        i32.const 4
        i32.lt_u
        select
        call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
        drop
        br 1 (;@1;)
      end
      block ;; label = @2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        local.get 2
        i32.ne
        br_if 0 (;@2;)
        i32.const 0
        call $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE
        drop
        i32.const 1
        return
      end
      i32.const 1
      local.set 0
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 2
      i32.const 6
      i32.add
      local.get 2
      local.get 2
      i32.const 0
      i32.lt_s
      select
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.const -6
      i32.add
      local.get 2
      i32.const 4
      i32.lt_u
      select
      call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
      drop
      i32.const 1
      return
    end
    local.get 0
  )
  (func $_ZN15strategy_hunter13predator_tick17hdd4a25299cdb0f63E (;17;) (type 2)
    (local i32 i32 i32 i32 i32 i32)
    block ;; label = @1
      call $_ZN15strategy_hunter11maybe_clone17h2c98f5d73388d033E
      br_if 0 (;@1;)
      i32.const 1
      local.set 0
      loop ;; label = @2
        block ;; label = @3
          i32.const 1
          i32.const 0
          call $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          i32.const 1
          i32.const 0
          i32.const 1048928
          call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
          drop
          block ;; label = @4
            i32.const 0
            i32.load offset=1048928
            i32.const 2
            i32.eq
            br_if 0 (;@4;)
            i32.const 1
            local.set 1
            i32.const 1
            i32.const -1
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.const 2
            i32.eq
            br_if 0 (;@4;)
            i32.const 0
            i32.const -1
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 2
            local.set 1
            i32.const 0
            i32.load offset=1048928
            i32.const 2
            i32.eq
            br_if 0 (;@4;)
            i32.const -1
            i32.const 0
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            block ;; label = @5
              i32.const 0
              i32.load offset=1048928
              i32.const 2
              i32.ne
              br_if 0 (;@5;)
              i32.const 3
              local.set 1
              br 1 (;@4;)
            end
            i32.const -1
            i32.const 1
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            block ;; label = @5
              i32.const 0
              i32.load offset=1048928
              i32.const 2
              i32.ne
              br_if 0 (;@5;)
              i32.const 4
              local.set 1
              br 1 (;@4;)
            end
            i32.const 0
            i32.const 1
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            block ;; label = @5
              i32.const 0
              i32.load offset=1048928
              i32.const 2
              i32.ne
              br_if 0 (;@5;)
              i32.const 5
              local.set 1
              br 1 (;@4;)
            end
            i32.const 1
            local.set 2
            block ;; label = @5
              block ;; label = @6
                block ;; label = @7
                  loop ;; label = @8
                    i32.const 0
                    local.set 1
                    loop ;; label = @9
                      local.get 2
                      local.get 1
                      i32.const 1048928
                      call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                      drop
                      block ;; label = @10
                        i32.const 0
                        i32.load offset=1048928
                        i32.const 2
                        i32.ne
                        br_if 0 (;@10;)
                        local.get 2
                        local.set 3
                        br 4 (;@6;)
                      end
                      local.get 2
                      local.get 1
                      i32.const -1
                      i32.add
                      local.tee 1
                      i32.add
                      br_if 0 (;@9;)
                    end
                    local.get 2
                    local.set 3
                    loop ;; label = @9
                      local.get 3
                      local.get 1
                      i32.const 1048928
                      call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                      drop
                      i32.const 0
                      i32.load offset=1048928
                      i32.const 2
                      i32.eq
                      br_if 3 (;@6;)
                      local.get 3
                      i32.const -1
                      i32.add
                      local.tee 3
                      br_if 0 (;@9;)
                    end
                    i32.const 0
                    local.set 4
                    loop ;; label = @9
                      local.get 4
                      local.get 1
                      i32.const 1048928
                      call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                      drop
                      i32.const 0
                      i32.load offset=1048928
                      i32.const 2
                      i32.eq
                      br_if 2 (;@7;)
                      local.get 1
                      i32.const 1
                      i32.add
                      local.set 1
                      local.get 2
                      local.get 4
                      i32.const -1
                      i32.add
                      local.tee 4
                      i32.add
                      br_if 0 (;@9;)
                    end
                    local.get 2
                    local.set 3
                    loop ;; label = @9
                      local.get 4
                      local.get 1
                      i32.const 1048928
                      call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                      drop
                      i32.const 0
                      i32.load offset=1048928
                      i32.const 2
                      i32.eq
                      br_if 2 (;@7;)
                      local.get 1
                      i32.const 1
                      i32.add
                      local.set 1
                      local.get 3
                      i32.const -1
                      i32.add
                      local.tee 3
                      br_if 0 (;@9;)
                    end
                    i32.const 0
                    local.set 5
                    loop ;; label = @9
                      local.get 4
                      local.get 5
                      i32.add
                      local.tee 3
                      local.get 1
                      i32.const 1048928
                      call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                      drop
                      i32.const 0
                      i32.load offset=1048928
                      i32.const 2
                      i32.eq
                      br_if 3 (;@6;)
                      local.get 2
                      local.get 5
                      i32.const 1
                      i32.add
                      local.tee 5
                      i32.ne
                      br_if 0 (;@9;)
                    end
                    local.get 4
                    local.get 5
                    i32.add
                    local.set 3
                    i32.const 0
                    local.set 5
                    loop ;; label = @9
                      local.get 3
                      local.get 1
                      local.get 5
                      i32.add
                      local.tee 4
                      i32.const 1048928
                      call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                      drop
                      i32.const 0
                      i32.load offset=1048928
                      i32.const 2
                      i32.eq
                      br_if 4 (;@5;)
                      local.get 3
                      i32.const 1
                      i32.add
                      local.set 3
                      local.get 2
                      local.get 5
                      i32.const -1
                      i32.add
                      local.tee 5
                      i32.add
                      br_if 0 (;@9;)
                    end
                    local.get 2
                    i32.const 1
                    i32.add
                    local.tee 2
                    i32.const 6
                    i32.ne
                    br_if 0 (;@8;)
                  end
                  call $_ZN15strategy_hunter11wander_step17ha908672c1f1e31deE
                  br 4 (;@3;)
                end
                local.get 4
                local.set 3
              end
              local.get 1
              local.set 4
            end
            i32.const 2
            call $_ZN15strategy_hunter4host16signal_broadcast17h575c2111c62c1639E
            drop
            local.get 3
            local.get 4
            call $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE
            br 1 (;@3;)
          end
          block ;; label = @4
            call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
            local.get 1
            i32.ne
            br_if 0 (;@4;)
            i32.const 0
            call $_ZN15strategy_hunter4host3hit17h862a6806f604c65cE
            drop
            br 1 (;@3;)
          end
          local.get 1
          call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
          i32.sub
          i32.const 6
          i32.rem_s
          local.tee 1
          i32.const 6
          i32.add
          local.get 1
          local.get 1
          i32.const 0
          i32.lt_s
          select
          local.tee 1
          i32.eqz
          br_if 0 (;@3;)
          local.get 1
          local.get 1
          i32.const -6
          i32.add
          local.get 1
          i32.const 4
          i32.lt_u
          select
          call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
          drop
        end
        local.get 0
        i32.const 1
        i32.and
        local.set 1
        i32.const 0
        local.set 0
        local.get 1
        br_if 0 (;@2;)
      end
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
    end
  )
  (func $_ZN4core9panicking18panic_bounds_check17hf4028f296c44236fE (;18;) (type 8) (param i32 i32 i32)
    (local i32 i64)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1048712
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    i32.const 1
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.tee 4
    local.get 3
    i64.extend_i32_u
    i64.or
    i64.store offset=40
    local.get 3
    local.get 4
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call $_ZN4core9panicking9panic_fmt17h808dbde205a89691E
    unreachable
  )
  (func $_ZN4core9panicking9panic_fmt17h808dbde205a89691E (;19;) (type 7) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 1
    i32.store16 offset=12
    local.get 2
    local.get 1
    i32.store offset=8
    local.get 2
    local.get 0
    i32.store offset=4
    local.get 2
    i32.const 4
    i32.add
    call $_RNvCsj4CZ6flxxfE_7___rustc17rust_begin_unwind
    unreachable
  )
  (func $_ZN4core3str5count14do_count_chars17haa2c4f188ad8cef2E (;20;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    block ;; label = @1
      block ;; label = @2
        local.get 1
        local.get 0
        i32.const 3
        i32.add
        i32.const -4
        i32.and
        local.tee 2
        local.get 0
        i32.sub
        local.tee 3
        i32.lt_u
        br_if 0 (;@2;)
        local.get 1
        local.get 3
        i32.sub
        local.tee 4
        i32.const 4
        i32.lt_u
        br_if 0 (;@2;)
        local.get 4
        i32.const 3
        i32.and
        local.set 5
        i32.const 0
        local.set 6
        i32.const 0
        local.set 1
        block ;; label = @3
          local.get 2
          local.get 0
          i32.eq
          local.tee 7
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          block ;; label = @4
            block ;; label = @5
              local.get 0
              local.get 2
              i32.sub
              local.tee 8
              i32.const -4
              i32.le_u
              br_if 0 (;@5;)
              i32.const 0
              local.set 9
              br 1 (;@4;)
            end
            i32.const 0
            local.set 9
            loop ;; label = @5
              local.get 1
              local.get 0
              local.get 9
              i32.add
              local.tee 2
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 1
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 2
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 3
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.set 1
              local.get 9
              i32.const 4
              i32.add
              local.tee 9
              br_if 0 (;@5;)
            end
          end
          local.get 7
          br_if 0 (;@3;)
          local.get 0
          local.get 9
          i32.add
          local.set 2
          loop ;; label = @4
            local.get 1
            local.get 2
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 2
            i32.const 1
            i32.add
            local.set 2
            local.get 8
            i32.const 1
            i32.add
            local.tee 8
            br_if 0 (;@4;)
          end
        end
        local.get 0
        local.get 3
        i32.add
        local.set 0
        block ;; label = @3
          local.get 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 4
          i32.const -4
          i32.and
          i32.add
          local.tee 2
          i32.load8_s
          i32.const -65
          i32.gt_s
          local.set 6
          local.get 5
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=1
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
          local.get 5
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=2
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
        end
        local.get 4
        i32.const 2
        i32.shr_u
        local.set 8
        local.get 6
        local.get 1
        i32.add
        local.set 3
        loop ;; label = @3
          local.get 0
          local.set 4
          local.get 8
          i32.eqz
          br_if 2 (;@1;)
          local.get 8
          i32.const 192
          local.get 8
          i32.const 192
          i32.lt_u
          select
          local.tee 6
          i32.const 3
          i32.and
          local.set 7
          local.get 6
          i32.const 2
          i32.shl
          local.set 5
          i32.const 0
          local.set 2
          block ;; label = @4
            local.get 8
            i32.const 4
            i32.lt_u
            br_if 0 (;@4;)
            local.get 4
            local.get 5
            i32.const 1008
            i32.and
            i32.add
            local.set 9
            i32.const 0
            local.set 2
            local.get 4
            local.set 1
            loop ;; label = @5
              local.get 1
              i32.const 12
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.const 8
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.const 4
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 2
              i32.add
              i32.add
              i32.add
              i32.add
              local.set 2
              local.get 1
              i32.const 16
              i32.add
              local.tee 1
              local.get 9
              i32.ne
              br_if 0 (;@5;)
            end
          end
          local.get 8
          local.get 6
          i32.sub
          local.set 8
          local.get 4
          local.get 5
          i32.add
          local.set 0
          local.get 2
          i32.const 8
          i32.shr_u
          i32.const 16711935
          i32.and
          local.get 2
          i32.const 16711935
          i32.and
          i32.add
          i32.const 65537
          i32.mul
          i32.const 16
          i32.shr_u
          local.get 3
          i32.add
          local.set 3
          local.get 7
          i32.eqz
          br_if 0 (;@3;)
        end
        local.get 4
        local.get 6
        i32.const 252
        i32.and
        i32.const 2
        i32.shl
        i32.add
        local.tee 2
        i32.load
        local.tee 1
        i32.const -1
        i32.xor
        i32.const 7
        i32.shr_u
        local.get 1
        i32.const 6
        i32.shr_u
        i32.or
        i32.const 16843009
        i32.and
        local.set 1
        block ;; label = @3
          local.get 7
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=4
          local.tee 0
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 0
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
          local.get 7
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=8
          local.tee 2
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 2
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
        end
        local.get 1
        i32.const 8
        i32.shr_u
        i32.const 459007
        i32.and
        local.get 1
        i32.const 16711935
        i32.and
        i32.add
        i32.const 65537
        i32.mul
        i32.const 16
        i32.shr_u
        local.get 3
        i32.add
        return
      end
      block ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        i32.const 0
        return
      end
      local.get 1
      i32.const 3
      i32.and
      local.set 9
      block ;; label = @2
        block ;; label = @3
          local.get 1
          i32.const 4
          i32.ge_u
          br_if 0 (;@3;)
          i32.const 0
          local.set 3
          i32.const 0
          local.set 2
          br 1 (;@2;)
        end
        local.get 1
        i32.const -4
        i32.and
        local.set 8
        i32.const 0
        local.set 3
        i32.const 0
        local.set 2
        loop ;; label = @3
          local.get 3
          local.get 0
          local.get 2
          i32.add
          local.tee 1
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 1
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 2
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 3
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 3
          local.get 8
          local.get 2
          i32.const 4
          i32.add
          local.tee 2
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 9
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      i32.add
      local.set 1
      loop ;; label = @2
        local.get 3
        local.get 1
        i32.load8_s
        i32.const -65
        i32.gt_s
        i32.add
        local.set 3
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 9
        i32.const -1
        i32.add
        local.tee 9
        br_if 0 (;@2;)
      end
    end
    local.get 3
  )
  (func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hacc3ad28f4de0a4dE (;21;) (type 0) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 8
    i32.add
    local.get 0
    i32.load
    local.get 2
    i32.const 22
    i32.add
    i32.const 10
    call $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb4e91fd13b3ed913E
    local.get 1
    i32.const 1
    i32.const 1
    i32.const 0
    local.get 2
    i32.load offset=8
    local.get 2
    i32.load offset=12
    call $_ZN4core3fmt9Formatter12pad_integral17hc6b3558773c79a2cE
    local.set 0
    local.get 2
    i32.const 32
    i32.add
    global.set $__stack_pointer
    local.get 0
  )
  (func $_ZN4core3fmt9Formatter12pad_integral17hc6b3558773c79a2cE (;22;) (type 9) (param i32 i32 i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i64)
    block ;; label = @1
      block ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        local.get 5
        i32.const 1
        i32.add
        local.set 6
        local.get 0
        i32.load offset=8
        local.set 7
        i32.const 45
        local.set 8
        br 1 (;@1;)
      end
      i32.const 43
      i32.const 1114112
      local.get 0
      i32.load offset=8
      local.tee 7
      i32.const 2097152
      i32.and
      local.tee 1
      select
      local.set 8
      local.get 1
      i32.const 21
      i32.shr_u
      local.get 5
      i32.add
      local.set 6
    end
    block ;; label = @1
      block ;; label = @2
        local.get 7
        i32.const 8388608
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        local.set 2
        br 1 (;@1;)
      end
      block ;; label = @2
        block ;; label = @3
          local.get 3
          i32.const 16
          i32.lt_u
          br_if 0 (;@3;)
          local.get 2
          local.get 3
          call $_ZN4core3str5count14do_count_chars17haa2c4f188ad8cef2E
          local.set 1
          br 1 (;@2;)
        end
        block ;; label = @3
          local.get 3
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          br 1 (;@2;)
        end
        local.get 3
        i32.const 3
        i32.and
        local.set 9
        block ;; label = @3
          block ;; label = @4
            local.get 3
            i32.const 4
            i32.ge_u
            br_if 0 (;@4;)
            i32.const 0
            local.set 1
            i32.const 0
            local.set 10
            br 1 (;@3;)
          end
          local.get 3
          i32.const 12
          i32.and
          local.set 11
          i32.const 0
          local.set 1
          i32.const 0
          local.set 10
          loop ;; label = @4
            local.get 1
            local.get 2
            local.get 10
            i32.add
            local.tee 12
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 1
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 2
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 3
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 11
            local.get 10
            i32.const 4
            i32.add
            local.tee 10
            i32.ne
            br_if 0 (;@4;)
          end
        end
        local.get 9
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 10
        i32.add
        local.set 12
        loop ;; label = @3
          local.get 1
          local.get 12
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 1
          local.get 12
          i32.const 1
          i32.add
          local.set 12
          local.get 9
          i32.const -1
          i32.add
          local.tee 9
          br_if 0 (;@3;)
        end
      end
      local.get 1
      local.get 6
      i32.add
      local.set 6
    end
    block ;; label = @1
      block ;; label = @2
        local.get 6
        local.get 0
        i32.load16_u offset=12
        local.tee 11
        i32.ge_u
        br_if 0 (;@2;)
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              local.get 7
              i32.const 16777216
              i32.and
              br_if 0 (;@5;)
              local.get 11
              local.get 6
              i32.sub
              local.set 13
              i32.const 0
              local.set 1
              i32.const 0
              local.set 11
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    local.get 7
                    i32.const 29
                    i32.shr_u
                    i32.const 3
                    i32.and
                    br_table 2 (;@6;) 0 (;@8;) 1 (;@7;) 0 (;@8;) 2 (;@6;)
                  end
                  local.get 13
                  local.set 11
                  br 1 (;@6;)
                end
                local.get 13
                i32.const 65534
                i32.and
                i32.const 1
                i32.shr_u
                local.set 11
              end
              local.get 7
              i32.const 2097151
              i32.and
              local.set 6
              local.get 0
              i32.load offset=4
              local.set 9
              local.get 0
              i32.load
              local.set 10
              loop ;; label = @6
                local.get 1
                i32.const 65535
                i32.and
                local.get 11
                i32.const 65535
                i32.and
                i32.ge_u
                br_if 2 (;@4;)
                i32.const 1
                local.set 12
                local.get 1
                i32.const 1
                i32.add
                local.set 1
                local.get 10
                local.get 6
                local.get 9
                i32.load offset=16
                call_indirect (type 0)
                i32.eqz
                br_if 0 (;@6;)
                br 5 (;@1;)
              end
            end
            local.get 0
            local.get 0
            i64.load offset=8 align=4
            local.tee 14
            i32.wrap_i64
            i32.const -1612709888
            i32.and
            i32.const 536870960
            i32.or
            i32.store offset=8
            i32.const 1
            local.set 12
            local.get 0
            i32.load
            local.tee 10
            local.get 0
            i32.load offset=4
            local.tee 9
            local.get 8
            local.get 2
            local.get 3
            call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
            br_if 3 (;@1;)
            i32.const 0
            local.set 1
            local.get 11
            local.get 6
            i32.sub
            i32.const 65535
            i32.and
            local.set 2
            loop ;; label = @5
              local.get 1
              i32.const 65535
              i32.and
              local.get 2
              i32.ge_u
              br_if 2 (;@3;)
              i32.const 1
              local.set 12
              local.get 1
              i32.const 1
              i32.add
              local.set 1
              local.get 10
              i32.const 48
              local.get 9
              i32.load offset=16
              call_indirect (type 0)
              i32.eqz
              br_if 0 (;@5;)
              br 4 (;@1;)
            end
          end
          i32.const 1
          local.set 12
          local.get 10
          local.get 9
          local.get 8
          local.get 2
          local.get 3
          call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
          br_if 2 (;@1;)
          local.get 10
          local.get 4
          local.get 5
          local.get 9
          i32.load offset=12
          call_indirect (type 1)
          br_if 2 (;@1;)
          i32.const 0
          local.set 1
          local.get 13
          local.get 11
          i32.sub
          i32.const 65535
          i32.and
          local.set 0
          loop ;; label = @4
            local.get 1
            i32.const 65535
            i32.and
            local.tee 2
            local.get 0
            i32.lt_u
            local.set 12
            local.get 2
            local.get 0
            i32.ge_u
            br_if 3 (;@1;)
            local.get 1
            i32.const 1
            i32.add
            local.set 1
            local.get 10
            local.get 6
            local.get 9
            i32.load offset=16
            call_indirect (type 0)
            i32.eqz
            br_if 0 (;@4;)
            br 3 (;@1;)
          end
        end
        i32.const 1
        local.set 12
        local.get 10
        local.get 4
        local.get 5
        local.get 9
        i32.load offset=12
        call_indirect (type 1)
        br_if 1 (;@1;)
        local.get 0
        local.get 14
        i64.store offset=8 align=4
        i32.const 0
        return
      end
      i32.const 1
      local.set 12
      local.get 0
      i32.load
      local.tee 1
      local.get 0
      i32.load offset=4
      local.tee 10
      local.get 8
      local.get 2
      local.get 3
      call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
      br_if 0 (;@1;)
      local.get 1
      local.get 4
      local.get 5
      local.get 10
      i32.load offset=12
      call_indirect (type 1)
      local.set 12
    end
    local.get 12
  )
  (func $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE (;23;) (type 10) (param i32 i32 i32 i32 i32) (result i32)
    block ;; label = @1
      local.get 2
      i32.const 1114112
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      local.get 1
      i32.load offset=16
      call_indirect (type 0)
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    block ;; label = @1
      local.get 3
      br_if 0 (;@1;)
      i32.const 0
      return
    end
    local.get 0
    local.get 3
    local.get 4
    local.get 1
    i32.load offset=12
    call_indirect (type 1)
  )
  (func $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb4e91fd13b3ed913E (;24;) (type 11) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    local.get 1
    local.set 4
    local.get 3
    local.set 5
    block ;; label = @1
      local.get 1
      i32.const 1000
      i32.lt_u
      br_if 0 (;@1;)
      local.get 2
      i32.const -4
      i32.add
      local.set 6
      local.get 3
      local.set 5
      local.get 1
      local.set 7
      loop ;; label = @2
        local.get 6
        local.get 5
        i32.add
        local.tee 8
        i32.const 1
        i32.add
        local.get 7
        local.get 7
        i32.const 10000
        i32.div_u
        local.tee 4
        i32.const 10000
        i32.mul
        i32.sub
        local.tee 9
        i32.const 65535
        i32.and
        i32.const 100
        i32.div_u
        local.tee 10
        i32.const 1
        i32.shl
        local.tee 11
        i32.const 1048729
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        local.get 11
        i32.const 1048728
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        i32.const 3
        i32.add
        local.get 9
        local.get 10
        i32.const 100
        i32.mul
        i32.sub
        i32.const 65535
        i32.and
        i32.const 1
        i32.shl
        local.tee 9
        i32.const 1048729
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        i32.const 2
        i32.add
        local.get 9
        i32.const 1048728
        i32.add
        i32.load8_u
        i32.store8
        local.get 5
        i32.const -4
        i32.add
        local.set 5
        local.get 7
        i32.const 9999999
        i32.gt_u
        local.set 8
        local.get 4
        local.set 7
        local.get 8
        br_if 0 (;@2;)
      end
    end
    block ;; label = @1
      block ;; label = @2
        local.get 4
        i32.const 9
        i32.gt_u
        br_if 0 (;@2;)
        local.get 4
        local.set 7
        br 1 (;@1;)
      end
      local.get 2
      local.get 5
      i32.add
      i32.const -1
      i32.add
      local.get 4
      local.get 4
      i32.const 65535
      i32.and
      i32.const 100
      i32.div_u
      local.tee 7
      i32.const 100
      i32.mul
      i32.sub
      i32.const 65535
      i32.and
      i32.const 1
      i32.shl
      local.tee 8
      i32.const 1048729
      i32.add
      i32.load8_u
      i32.store8
      local.get 2
      local.get 5
      i32.const -2
      i32.add
      local.tee 5
      i32.add
      local.get 8
      i32.const 1048728
      i32.add
      i32.load8_u
      i32.store8
    end
    block ;; label = @1
      block ;; label = @2
        local.get 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 7
        i32.eqz
        br_if 1 (;@1;)
      end
      local.get 2
      local.get 5
      i32.const -1
      i32.add
      local.tee 5
      i32.add
      local.get 7
      i32.const 1
      i32.shl
      i32.const 30
      i32.and
      i32.const 1048729
      i32.add
      i32.load8_u
      i32.store8
    end
    local.get 0
    local.get 3
    local.get 5
    i32.sub
    i32.store offset=4
    local.get 0
    local.get 2
    local.get 5
    i32.add
    i32.store
  )
  (data $.rodata (;0;) (i32.const 1048576) "\01\00\00\00\00\00\00\00\01\00\00\00\ff\ff\ff\ff\00\00\00\00\ff\ff\ff\ff\ff\ff\ff\ff\00\00\00\00\ff\ff\ff\ff\01\00\00\00\00\00\00\00\01\00\00\00hunter/src/lib.rs\00\00\000\00\10\00\11\00\00\00\cf\00\00\00\1c\00\00\00index out of bounds: the len is  but the index is \00\00T\00\10\00 \00\00\00t\00\10\00\12\00\00\0000010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899")
  (@producers
    (language "Rust" "")
    (processed-by "rustc" "1.90.0 (1159e78c4 2025-09-14)")
  )
  (@custom "target_features" (after data) "\08+\0bbulk-memory+\0fbulk-memory-opt+\16call-indirect-overlong+\0amultivalue+\0fmutable-globals+\13nontrapping-fptoint+\0freference-types+\08sign-ext")
)

"#;

pub(crate) const HAWK: &str = r#"
(module $strategy_hawk.wasm
  (type (;0;) (func (param i32 i32) (result i32)))
  (type (;1;) (func (param i32 i32 i32) (result i32)))
  (type (;2;) (func))
  (type (;3;) (func (result i32)))
  (type (;4;) (func (param i32) (result i32)))
  (type (;5;) (func (result i64)))
  (type (;6;) (func (param i32)))
  (type (;7;) (func (param i32 i32)))
  (type (;8;) (func (param i32 i32 i32)))
  (type (;9;) (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (type (;10;) (func (param i32 i32 i32 i32 i32) (result i32)))
  (type (;11;) (func (param i32 i32 i32 i32)))
  (import "terrarium" "sleep" (func $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E (;0;) (type 2)))
  (import "terrarium" "facing" (func $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E (;1;) (type 3)))
  (import "terrarium" "move" (func $_ZN15strategy_hunter4host4step17h54526a67a501102bE (;2;) (type 4)))
  (import "terrarium" "rotate" (func $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE (;3;) (type 4)))
  (import "terrarium" "random_byte" (func $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E (;4;) (type 3)))
  (import "terrarium" "energy" (func $_ZN15strategy_hunter4host6energy17h025e496adf3b9fedE (;5;) (type 5)))
  (import "terrarium" "sense" (func $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E (;6;) (type 1)))
  (import "terrarium" "spawn" (func $_ZN15strategy_hunter4host5spawn17h1c054dc162b77db4E (;7;) (type 0)))
  (import "terrarium" "eat" (func $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE (;8;) (type 4)))
  (import "terrarium" "recv" (func $_ZN15strategy_hunter4host4recv17h0668d301ceeeb97fE (;9;) (type 4)))
  (import "terrarium" "pos_x" (func $_ZN15strategy_hunter4host5pos_x17h83d553ce0faa30ccE (;10;) (type 3)))
  (import "terrarium" "pos_y" (func $_ZN15strategy_hunter4host5pos_y17h042d3565584392bdE (;11;) (type 3)))
  (table (;0;) 2 2 funcref)
  (memory (;0;) 17)
  (global $__stack_pointer (;0;) (mut i32) i32.const 1048576)
  (global (;1;) i32 i32.const 1048988)
  (global (;2;) i32 i32.const 1048992)
  (export "memory" (memory 0))
  (export "main" (func $main))
  (export "__data_end" (global 1))
  (export "__heap_base" (global 2))
  (elem (;0;) (i32.const 1) func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hacc3ad28f4de0a4dE)
  (func $main (;12;) (type 2)
    loop ;; label = @1
      call $_ZN15strategy_hunter14scavenger_tick17haaa34e6990b5ca87E
      br 0 (;@1;)
    end
  )
  (func $_RNvCsj4CZ6flxxfE_7___rustc17rust_begin_unwind (;13;) (type 6) (param i32)
    call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
    loop ;; label = @1
      br 0 (;@1;)
    end
  )
  (func $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE (;14;) (type 7) (param i32 i32)
    (local i32)
    i32.const 0
    local.set 2
    block ;; label = @1
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            local.get 0
            i32.const 1
            i32.add
            br_table 1 (;@3;) 0 (;@4;) 2 (;@2;) 3 (;@1;)
          end
          i32.const 2
          i32.const 5
          i32.const 0
          local.get 1
          i32.const 1
          i32.eq
          select
          local.get 1
          i32.const -1
          i32.eq
          select
          local.set 2
          br 2 (;@1;)
        end
        local.get 1
        i32.const 1
        i32.eq
        i32.const 2
        i32.shl
        i32.const 3
        local.get 1
        select
        local.set 2
        br 1 (;@1;)
      end
      local.get 1
      i32.const -1
      i32.eq
      local.set 2
    end
    block ;; label = @1
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 1
      i32.const 6
      i32.add
      local.get 1
      local.get 1
      i32.const 0
      i32.lt_s
      select
      br_if 0 (;@1;)
      i32.const 0
      call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
      drop
      return
    end
    block ;; label = @1
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 2
      i32.const 6
      i32.add
      local.get 2
      local.get 2
      i32.const 0
      i32.lt_s
      select
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.const -6
      i32.add
      local.get 2
      i32.const 4
      i32.lt_u
      select
      call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
      drop
    end
  )
  (func $_ZN15strategy_hunter11maybe_clone17h2c98f5d73388d033E (;15;) (type 3) (result i32)
    (local i32 i32 i32)
    i32.const 0
    local.set 0
    block ;; label = @1
      call $_ZN15strategy_hunter4host6energy17h025e496adf3b9fedE
      i64.const 5000001
      i64.lt_s
      br_if 0 (;@1;)
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
            i32.const 6
            i32.rem_s
            local.tee 1
            i32.const -1
            i32.le_s
            br_if 0 (;@4;)
            local.get 1
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 1 (;@3;)
            i32.const 1
            i32.const -5
            local.get 1
            i32.const 5
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 2
            i32.const -4
            local.get 1
            i32.const 4
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 3
            i32.const -3
            local.get 1
            i32.const 3
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            i32.const 4
            i32.const -2
            local.get 1
            i32.const 2
            i32.lt_u
            select
            local.get 1
            i32.add
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 0
            i32.const 1048576
            i32.add
            i32.load
            local.get 0
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            local.set 0
            i32.const 0
            i32.load offset=1048928
            i32.eqz
            br_if 2 (;@2;)
            local.get 1
            i32.const -1
            i32.add
            i32.const 5
            local.get 1
            select
            local.tee 2
            i32.const 3
            i32.shl
            local.tee 1
            i32.const 1048576
            i32.add
            i32.load
            local.get 1
            i32.const 1048580
            i32.add
            i32.load
            i32.const 1048928
            call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
            drop
            i32.const 0
            i32.load offset=1048928
            br_if 3 (;@1;)
            br 2 (;@2;)
          end
          local.get 1
          i32.const 6
          i32.const 1048644
          call $_ZN4core9panicking18panic_bounds_check17hf4028f296c44236fE
          unreachable
        end
        local.get 1
        local.set 2
      end
      block ;; label = @2
        block ;; label = @3
          call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
          local.get 2
          i32.ne
          br_if 0 (;@3;)
          i32.const 0
          i32.const 2000000
          call $_ZN15strategy_hunter4host5spawn17h1c054dc162b77db4E
          drop
          br 1 (;@2;)
        end
        local.get 2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        i32.sub
        i32.const 6
        i32.rem_s
        local.tee 1
        i32.const 6
        i32.add
        local.get 1
        local.get 1
        i32.const 0
        i32.lt_s
        select
        local.tee 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 1
        local.get 1
        i32.const -6
        i32.add
        local.get 1
        i32.const 4
        i32.lt_u
        select
        call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
        drop
      end
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
      i32.const 1
      local.set 0
    end
    local.get 0
  )
  (func $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E (;16;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i64 i32 i32 i32 i64 i32 i32)
    i32.const 0
    local.set 2
    i32.const 1
    i32.const 0
    i32.const 1048928
    call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
    drop
    block ;; label = @1
      block ;; label = @2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 1
        local.set 2
        i32.const 1
        i32.const -1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        i32.const -1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 2
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const -1
        i32.const 0
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 3
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const -1
        i32.const 1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 4
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        i32.const 1
        i32.const 1048928
        call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
        drop
        i32.const 5
        local.set 2
        i32.const 0
        i32.load offset=1048928
        local.tee 3
        i32.const 4
        i32.eq
        br_if 0 (;@2;)
        local.get 0
        local.get 3
        i32.const 3
        i32.eq
        i32.and
        br_if 0 (;@2;)
        block ;; label = @3
          block ;; label = @4
            local.get 0
            br_if 0 (;@4;)
            i64.const 0
            local.set 4
            i32.const 0
            local.set 5
            i32.const 0
            local.set 6
            i32.const 0
            local.set 7
            i32.const 1
            local.set 0
            loop ;; label = @5
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 0
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 0
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const -1
                i32.add
                local.tee 2
                i32.add
                br_if 0 (;@6;)
              end
              local.get 0
              local.set 3
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 3
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 2
                i32.const 1
                i32.add
                local.set 2
                local.get 0
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                i32.add
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 2
                  local.set 6
                  local.get 3
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const 1
                i32.add
                local.tee 2
                i32.ne
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              loop ;; label = @6
                local.get 3
                local.get 2
                i32.add
                local.tee 9
                local.get 0
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 0
                  local.set 6
                  local.get 9
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 0
                local.get 2
                i32.const 1
                i32.add
                local.tee 2
                i32.ne
                br_if 0 (;@6;)
              end
              i32.const 0
              local.set 2
              local.get 0
              local.set 3
              loop ;; label = @6
                local.get 2
                local.get 3
                i32.const 1048928
                call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
                drop
                block ;; label = @7
                  i32.const 0
                  i32.load offset=1048928
                  i32.const 4
                  i32.ne
                  br_if 0 (;@7;)
                  i32.const 0
                  i64.load offset=1048936 align=4
                  local.tee 8
                  local.get 4
                  i64.le_s
                  br_if 0 (;@7;)
                  i32.const 1
                  local.set 7
                  local.get 3
                  local.set 6
                  local.get 2
                  local.set 5
                  local.get 8
                  local.set 4
                end
                local.get 2
                i32.const 1
                i32.add
                local.set 2
                local.get 3
                i32.const -1
                i32.add
                local.tee 3
                br_if 0 (;@6;)
              end
              local.get 0
              i32.const 1
              i32.add
              local.tee 0
              i32.const 6
              i32.ne
              br_if 0 (;@5;)
              br 2 (;@3;)
            end
          end
          i64.const 0
          local.set 4
          i32.const 0
          local.set 5
          i32.const 0
          local.set 6
          i32.const 0
          local.set 7
          i32.const 1
          local.set 0
          loop ;; label = @4
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 0
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 3
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 3
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 0
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const -1
              i32.add
              local.tee 2
              i32.add
              br_if 0 (;@5;)
            end
            local.get 0
            local.set 3
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 3
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 2
              i32.const 1
              i32.add
              local.set 2
              local.get 0
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              i32.add
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 2
                local.set 6
                local.get 3
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const 1
              i32.add
              local.tee 2
              i32.ne
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            loop ;; label = @5
              local.get 3
              local.get 2
              i32.add
              local.tee 9
              local.get 0
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 10
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 10
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 0
                local.set 6
                local.get 9
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 0
              local.get 2
              i32.const 1
              i32.add
              local.tee 2
              i32.ne
              br_if 0 (;@5;)
            end
            i32.const 0
            local.set 2
            local.get 0
            local.set 3
            loop ;; label = @5
              local.get 2
              local.get 3
              i32.const 1048928
              call $_ZN15strategy_hunter4host5sense17hc58ae7581ba11934E
              drop
              i32.const 0
              i64.load offset=1048936 align=4
              local.set 8
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    i32.const 0
                    i32.load offset=1048928
                    local.tee 9
                    i32.const 3
                    i32.eq
                    br_if 0 (;@8;)
                    local.get 9
                    i32.const 4
                    i32.ne
                    br_if 2 (;@6;)
                    local.get 8
                    local.get 4
                    i64.gt_s
                    br_if 1 (;@7;)
                    br 2 (;@6;)
                  end
                  local.get 8
                  local.get 4
                  i64.le_s
                  br_if 1 (;@6;)
                end
                i32.const 1
                local.set 7
                local.get 3
                local.set 6
                local.get 2
                local.set 5
                local.get 8
                local.set 4
              end
              local.get 2
              i32.const 1
              i32.add
              local.set 2
              local.get 3
              i32.const -1
              i32.add
              local.tee 3
              br_if 0 (;@5;)
            end
            local.get 0
            i32.const 1
            i32.add
            local.tee 0
            i32.const 6
            i32.ne
            br_if 0 (;@4;)
          end
        end
        block ;; label = @3
          block ;; label = @4
            local.get 7
            i32.const 1
            i32.and
            br_if 0 (;@4;)
            i32.const 0
            local.set 0
            local.get 1
            i32.eqz
            br_if 3 (;@1;)
            call $_ZN15strategy_hunter4host11random_byte17h0018bc62b5018da3E
            local.set 2
            call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
            local.get 2
            i32.const 6
            i32.rem_s
            local.tee 2
            i32.ne
            br_if 1 (;@3;)
            i32.const 0
            call $_ZN15strategy_hunter4host4step17h54526a67a501102bE
            drop
            i32.const 1
            return
          end
          local.get 5
          local.get 6
          call $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE
          i32.const 1
          return
        end
        i32.const 1
        local.set 0
        local.get 2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        i32.sub
        i32.const 6
        i32.rem_s
        local.tee 2
        i32.const 6
        i32.add
        local.get 2
        local.get 2
        i32.const 0
        i32.lt_s
        select
        local.tee 2
        i32.eqz
        br_if 1 (;@1;)
        local.get 2
        local.get 2
        i32.const -6
        i32.add
        local.get 2
        i32.const 4
        i32.lt_u
        select
        call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
        drop
        br 1 (;@1;)
      end
      block ;; label = @2
        call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
        local.get 2
        i32.ne
        br_if 0 (;@2;)
        i32.const 0
        call $_ZN15strategy_hunter4host3eat17h998f6d8089554deaE
        drop
        i32.const 1
        return
      end
      i32.const 1
      local.set 0
      local.get 2
      call $_ZN15strategy_hunter4host6facing17hf13dd96bc0305814E
      i32.sub
      i32.const 6
      i32.rem_s
      local.tee 2
      i32.const 6
      i32.add
      local.get 2
      local.get 2
      i32.const 0
      i32.lt_s
      select
      local.tee 2
      i32.eqz
      br_if 0 (;@1;)
      local.get 2
      local.get 2
      i32.const -6
      i32.add
      local.get 2
      i32.const 4
      i32.lt_u
      select
      call $_ZN15strategy_hunter4host6rotate17h9dd28ac5cb3b3fdcE
      drop
      i32.const 1
      return
    end
    local.get 0
  )
  (func $_ZN15strategy_hunter14scavenger_tick17haaa34e6990b5ca87E (;17;) (type 2)
    (local i32)
    block ;; label = @1
      call $_ZN15strategy_hunter11maybe_clone17h2c98f5d73388d033E
      br_if 0 (;@1;)
      block ;; label = @2
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              i32.const 1048952
              call $_ZN15strategy_hunter4host4recv17h0668d301ceeeb97fE
              i32.eqz
              br_if 0 (;@5;)
              i32.const 0
              i32.load offset=1048964 align=1
              i32.const 1
              i32.eq
              br_if 1 (;@4;)
            end
            i32.const 1
            i32.const 1
            call $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E
            br_if 1 (;@3;)
            br 2 (;@2;)
          end
          i32.const 0
          i32.load offset=1048960 align=1
          local.set 0
          i32.const 0
          i32.load offset=1048956 align=1
          call $_ZN15strategy_hunter4host5pos_x17h83d553ce0faa30ccE
          i32.sub
          local.get 0
          call $_ZN15strategy_hunter4host5pos_y17h042d3565584392bdE
          i32.sub
          call $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE
        end
        block ;; label = @3
          i32.const 1048952
          call $_ZN15strategy_hunter4host4recv17h0668d301ceeeb97fE
          i32.eqz
          br_if 0 (;@3;)
          i32.const 0
          i32.load offset=1048964 align=1
          i32.const 1
          i32.ne
          br_if 0 (;@3;)
          i32.const 0
          i32.load offset=1048960 align=1
          local.set 0
          i32.const 0
          i32.load offset=1048956 align=1
          call $_ZN15strategy_hunter4host5pos_x17h83d553ce0faa30ccE
          i32.sub
          local.get 0
          call $_ZN15strategy_hunter4host5pos_y17h042d3565584392bdE
          i32.sub
          call $_ZN15strategy_hunter11step_toward17h164826cb30857a9dE
          br 1 (;@2;)
        end
        i32.const 1
        i32.const 1
        call $_ZN15strategy_hunter9seek_food17hdc63cb46d6416604E
        drop
      end
      call $_ZN15strategy_hunter4host5sleep17h937936acb95e37b2E
    end
  )
  (func $_ZN4core9panicking18panic_bounds_check17hf4028f296c44236fE (;18;) (type 8) (param i32 i32 i32)
    (local i32 i64)
    global.get $__stack_pointer
    i32.const 48
    i32.sub
    local.tee 3
    global.set $__stack_pointer
    local.get 3
    local.get 1
    i32.store offset=4
    local.get 3
    local.get 0
    i32.store
    local.get 3
    i32.const 2
    i32.store offset=12
    local.get 3
    i32.const 1048712
    i32.store offset=8
    local.get 3
    i64.const 2
    i64.store offset=20 align=4
    local.get 3
    i32.const 1
    i64.extend_i32_u
    i64.const 32
    i64.shl
    local.tee 4
    local.get 3
    i64.extend_i32_u
    i64.or
    i64.store offset=40
    local.get 3
    local.get 4
    local.get 3
    i32.const 4
    i32.add
    i64.extend_i32_u
    i64.or
    i64.store offset=32
    local.get 3
    local.get 3
    i32.const 32
    i32.add
    i32.store offset=16
    local.get 3
    i32.const 8
    i32.add
    local.get 2
    call $_ZN4core9panicking9panic_fmt17h808dbde205a89691E
    unreachable
  )
  (func $_ZN4core9panicking9panic_fmt17h808dbde205a89691E (;19;) (type 7) (param i32 i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 16
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 1
    i32.store16 offset=12
    local.get 2
    local.get 1
    i32.store offset=8
    local.get 2
    local.get 0
    i32.store offset=4
    local.get 2
    i32.const 4
    i32.add
    call $_RNvCsj4CZ6flxxfE_7___rustc17rust_begin_unwind
    unreachable
  )
  (func $_ZN4core3str5count14do_count_chars17haa2c4f188ad8cef2E (;20;) (type 0) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    block ;; label = @1
      block ;; label = @2
        local.get 1
        local.get 0
        i32.const 3
        i32.add
        i32.const -4
        i32.and
        local.tee 2
        local.get 0
        i32.sub
        local.tee 3
        i32.lt_u
        br_if 0 (;@2;)
        local.get 1
        local.get 3
        i32.sub
        local.tee 4
        i32.const 4
        i32.lt_u
        br_if 0 (;@2;)
        local.get 4
        i32.const 3
        i32.and
        local.set 5
        i32.const 0
        local.set 6
        i32.const 0
        local.set 1
        block ;; label = @3
          local.get 2
          local.get 0
          i32.eq
          local.tee 7
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          block ;; label = @4
            block ;; label = @5
              local.get 0
              local.get 2
              i32.sub
              local.tee 8
              i32.const -4
              i32.le_u
              br_if 0 (;@5;)
              i32.const 0
              local.set 9
              br 1 (;@4;)
            end
            i32.const 0
            local.set 9
            loop ;; label = @5
              local.get 1
              local.get 0
              local.get 9
              i32.add
              local.tee 2
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 1
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 2
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.get 2
              i32.const 3
              i32.add
              i32.load8_s
              i32.const -65
              i32.gt_s
              i32.add
              local.set 1
              local.get 9
              i32.const 4
              i32.add
              local.tee 9
              br_if 0 (;@5;)
            end
          end
          local.get 7
          br_if 0 (;@3;)
          local.get 0
          local.get 9
          i32.add
          local.set 2
          loop ;; label = @4
            local.get 1
            local.get 2
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 2
            i32.const 1
            i32.add
            local.set 2
            local.get 8
            i32.const 1
            i32.add
            local.tee 8
            br_if 0 (;@4;)
          end
        end
        local.get 0
        local.get 3
        i32.add
        local.set 0
        block ;; label = @3
          local.get 5
          i32.eqz
          br_if 0 (;@3;)
          local.get 0
          local.get 4
          i32.const -4
          i32.and
          i32.add
          local.tee 2
          i32.load8_s
          i32.const -65
          i32.gt_s
          local.set 6
          local.get 5
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=1
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
          local.get 5
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 6
          local.get 2
          i32.load8_s offset=2
          i32.const -65
          i32.gt_s
          i32.add
          local.set 6
        end
        local.get 4
        i32.const 2
        i32.shr_u
        local.set 8
        local.get 6
        local.get 1
        i32.add
        local.set 3
        loop ;; label = @3
          local.get 0
          local.set 4
          local.get 8
          i32.eqz
          br_if 2 (;@1;)
          local.get 8
          i32.const 192
          local.get 8
          i32.const 192
          i32.lt_u
          select
          local.tee 6
          i32.const 3
          i32.and
          local.set 7
          local.get 6
          i32.const 2
          i32.shl
          local.set 5
          i32.const 0
          local.set 2
          block ;; label = @4
            local.get 8
            i32.const 4
            i32.lt_u
            br_if 0 (;@4;)
            local.get 4
            local.get 5
            i32.const 1008
            i32.and
            i32.add
            local.set 9
            i32.const 0
            local.set 2
            local.get 4
            local.set 1
            loop ;; label = @5
              local.get 1
              i32.const 12
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.const 8
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.const 4
              i32.add
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 1
              i32.load
              local.tee 0
              i32.const -1
              i32.xor
              i32.const 7
              i32.shr_u
              local.get 0
              i32.const 6
              i32.shr_u
              i32.or
              i32.const 16843009
              i32.and
              local.get 2
              i32.add
              i32.add
              i32.add
              i32.add
              local.set 2
              local.get 1
              i32.const 16
              i32.add
              local.tee 1
              local.get 9
              i32.ne
              br_if 0 (;@5;)
            end
          end
          local.get 8
          local.get 6
          i32.sub
          local.set 8
          local.get 4
          local.get 5
          i32.add
          local.set 0
          local.get 2
          i32.const 8
          i32.shr_u
          i32.const 16711935
          i32.and
          local.get 2
          i32.const 16711935
          i32.and
          i32.add
          i32.const 65537
          i32.mul
          i32.const 16
          i32.shr_u
          local.get 3
          i32.add
          local.set 3
          local.get 7
          i32.eqz
          br_if 0 (;@3;)
        end
        local.get 4
        local.get 6
        i32.const 252
        i32.and
        i32.const 2
        i32.shl
        i32.add
        local.tee 2
        i32.load
        local.tee 1
        i32.const -1
        i32.xor
        i32.const 7
        i32.shr_u
        local.get 1
        i32.const 6
        i32.shr_u
        i32.or
        i32.const 16843009
        i32.and
        local.set 1
        block ;; label = @3
          local.get 7
          i32.const 1
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=4
          local.tee 0
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 0
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
          local.get 7
          i32.const 2
          i32.eq
          br_if 0 (;@3;)
          local.get 2
          i32.load offset=8
          local.tee 2
          i32.const -1
          i32.xor
          i32.const 7
          i32.shr_u
          local.get 2
          i32.const 6
          i32.shr_u
          i32.or
          i32.const 16843009
          i32.and
          local.get 1
          i32.add
          local.set 1
        end
        local.get 1
        i32.const 8
        i32.shr_u
        i32.const 459007
        i32.and
        local.get 1
        i32.const 16711935
        i32.and
        i32.add
        i32.const 65537
        i32.mul
        i32.const 16
        i32.shr_u
        local.get 3
        i32.add
        return
      end
      block ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        i32.const 0
        return
      end
      local.get 1
      i32.const 3
      i32.and
      local.set 9
      block ;; label = @2
        block ;; label = @3
          local.get 1
          i32.const 4
          i32.ge_u
          br_if 0 (;@3;)
          i32.const 0
          local.set 3
          i32.const 0
          local.set 2
          br 1 (;@2;)
        end
        local.get 1
        i32.const -4
        i32.and
        local.set 8
        i32.const 0
        local.set 3
        i32.const 0
        local.set 2
        loop ;; label = @3
          local.get 3
          local.get 0
          local.get 2
          i32.add
          local.tee 1
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 1
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 2
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.get 1
          i32.const 3
          i32.add
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 3
          local.get 8
          local.get 2
          i32.const 4
          i32.add
          local.tee 2
          i32.ne
          br_if 0 (;@3;)
        end
      end
      local.get 9
      i32.eqz
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      i32.add
      local.set 1
      loop ;; label = @2
        local.get 3
        local.get 1
        i32.load8_s
        i32.const -65
        i32.gt_s
        i32.add
        local.set 3
        local.get 1
        i32.const 1
        i32.add
        local.set 1
        local.get 9
        i32.const -1
        i32.add
        local.tee 9
        br_if 0 (;@2;)
      end
    end
    local.get 3
  )
  (func $_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17hacc3ad28f4de0a4dE (;21;) (type 0) (param i32 i32) (result i32)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 2
    global.set $__stack_pointer
    local.get 2
    i32.const 8
    i32.add
    local.get 0
    i32.load
    local.get 2
    i32.const 22
    i32.add
    i32.const 10
    call $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb4e91fd13b3ed913E
    local.get 1
    i32.const 1
    i32.const 1
    i32.const 0
    local.get 2
    i32.load offset=8
    local.get 2
    i32.load offset=12
    call $_ZN4core3fmt9Formatter12pad_integral17hc6b3558773c79a2cE
    local.set 0
    local.get 2
    i32.const 32
    i32.add
    global.set $__stack_pointer
    local.get 0
  )
  (func $_ZN4core3fmt9Formatter12pad_integral17hc6b3558773c79a2cE (;22;) (type 9) (param i32 i32 i32 i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i64)
    block ;; label = @1
      block ;; label = @2
        local.get 1
        br_if 0 (;@2;)
        local.get 5
        i32.const 1
        i32.add
        local.set 6
        local.get 0
        i32.load offset=8
        local.set 7
        i32.const 45
        local.set 8
        br 1 (;@1;)
      end
      i32.const 43
      i32.const 1114112
      local.get 0
      i32.load offset=8
      local.tee 7
      i32.const 2097152
      i32.and
      local.tee 1
      select
      local.set 8
      local.get 1
      i32.const 21
      i32.shr_u
      local.get 5
      i32.add
      local.set 6
    end
    block ;; label = @1
      block ;; label = @2
        local.get 7
        i32.const 8388608
        i32.and
        br_if 0 (;@2;)
        i32.const 0
        local.set 2
        br 1 (;@1;)
      end
      block ;; label = @2
        block ;; label = @3
          local.get 3
          i32.const 16
          i32.lt_u
          br_if 0 (;@3;)
          local.get 2
          local.get 3
          call $_ZN4core3str5count14do_count_chars17haa2c4f188ad8cef2E
          local.set 1
          br 1 (;@2;)
        end
        block ;; label = @3
          local.get 3
          br_if 0 (;@3;)
          i32.const 0
          local.set 1
          br 1 (;@2;)
        end
        local.get 3
        i32.const 3
        i32.and
        local.set 9
        block ;; label = @3
          block ;; label = @4
            local.get 3
            i32.const 4
            i32.ge_u
            br_if 0 (;@4;)
            i32.const 0
            local.set 1
            i32.const 0
            local.set 10
            br 1 (;@3;)
          end
          local.get 3
          i32.const 12
          i32.and
          local.set 11
          i32.const 0
          local.set 1
          i32.const 0
          local.set 10
          loop ;; label = @4
            local.get 1
            local.get 2
            local.get 10
            i32.add
            local.tee 12
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 1
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 2
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.get 12
            i32.const 3
            i32.add
            i32.load8_s
            i32.const -65
            i32.gt_s
            i32.add
            local.set 1
            local.get 11
            local.get 10
            i32.const 4
            i32.add
            local.tee 10
            i32.ne
            br_if 0 (;@4;)
          end
        end
        local.get 9
        i32.eqz
        br_if 0 (;@2;)
        local.get 2
        local.get 10
        i32.add
        local.set 12
        loop ;; label = @3
          local.get 1
          local.get 12
          i32.load8_s
          i32.const -65
          i32.gt_s
          i32.add
          local.set 1
          local.get 12
          i32.const 1
          i32.add
          local.set 12
          local.get 9
          i32.const -1
          i32.add
          local.tee 9
          br_if 0 (;@3;)
        end
      end
      local.get 1
      local.get 6
      i32.add
      local.set 6
    end
    block ;; label = @1
      block ;; label = @2
        local.get 6
        local.get 0
        i32.load16_u offset=12
        local.tee 11
        i32.ge_u
        br_if 0 (;@2;)
        block ;; label = @3
          block ;; label = @4
            block ;; label = @5
              local.get 7
              i32.const 16777216
              i32.and
              br_if 0 (;@5;)
              local.get 11
              local.get 6
              i32.sub
              local.set 13
              i32.const 0
              local.set 1
              i32.const 0
              local.set 11
              block ;; label = @6
                block ;; label = @7
                  block ;; label = @8
                    local.get 7
                    i32.const 29
                    i32.shr_u
                    i32.const 3
                    i32.and
                    br_table 2 (;@6;) 0 (;@8;) 1 (;@7;) 0 (;@8;) 2 (;@6;)
                  end
                  local.get 13
                  local.set 11
                  br 1 (;@6;)
                end
                local.get 13
                i32.const 65534
                i32.and
                i32.const 1
                i32.shr_u
                local.set 11
              end
              local.get 7
              i32.const 2097151
              i32.and
              local.set 6
              local.get 0
              i32.load offset=4
              local.set 9
              local.get 0
              i32.load
              local.set 10
              loop ;; label = @6
                local.get 1
                i32.const 65535
                i32.and
                local.get 11
                i32.const 65535
                i32.and
                i32.ge_u
                br_if 2 (;@4;)
                i32.const 1
                local.set 12
                local.get 1
                i32.const 1
                i32.add
                local.set 1
                local.get 10
                local.get 6
                local.get 9
                i32.load offset=16
                call_indirect (type 0)
                i32.eqz
                br_if 0 (;@6;)
                br 5 (;@1;)
              end
            end
            local.get 0
            local.get 0
            i64.load offset=8 align=4
            local.tee 14
            i32.wrap_i64
            i32.const -1612709888
            i32.and
            i32.const 536870960
            i32.or
            i32.store offset=8
            i32.const 1
            local.set 12
            local.get 0
            i32.load
            local.tee 10
            local.get 0
            i32.load offset=4
            local.tee 9
            local.get 8
            local.get 2
            local.get 3
            call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
            br_if 3 (;@1;)
            i32.const 0
            local.set 1
            local.get 11
            local.get 6
            i32.sub
            i32.const 65535
            i32.and
            local.set 2
            loop ;; label = @5
              local.get 1
              i32.const 65535
              i32.and
              local.get 2
              i32.ge_u
              br_if 2 (;@3;)
              i32.const 1
              local.set 12
              local.get 1
              i32.const 1
              i32.add
              local.set 1
              local.get 10
              i32.const 48
              local.get 9
              i32.load offset=16
              call_indirect (type 0)
              i32.eqz
              br_if 0 (;@5;)
              br 4 (;@1;)
            end
          end
          i32.const 1
          local.set 12
          local.get 10
          local.get 9
          local.get 8
          local.get 2
          local.get 3
          call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
          br_if 2 (;@1;)
          local.get 10
          local.get 4
          local.get 5
          local.get 9
          i32.load offset=12
          call_indirect (type 1)
          br_if 2 (;@1;)
          i32.const 0
          local.set 1
          local.get 13
          local.get 11
          i32.sub
          i32.const 65535
          i32.and
          local.set 0
          loop ;; label = @4
            local.get 1
            i32.const 65535
            i32.and
            local.tee 2
            local.get 0
            i32.lt_u
            local.set 12
            local.get 2
            local.get 0
            i32.ge_u
            br_if 3 (;@1;)
            local.get 1
            i32.const 1
            i32.add
            local.set 1
            local.get 10
            local.get 6
            local.get 9
            i32.load offset=16
            call_indirect (type 0)
            i32.eqz
            br_if 0 (;@4;)
            br 3 (;@1;)
          end
        end
        i32.const 1
        local.set 12
        local.get 10
        local.get 4
        local.get 5
        local.get 9
        i32.load offset=12
        call_indirect (type 1)
        br_if 1 (;@1;)
        local.get 0
        local.get 14
        i64.store offset=8 align=4
        i32.const 0
        return
      end
      i32.const 1
      local.set 12
      local.get 0
      i32.load
      local.tee 1
      local.get 0
      i32.load offset=4
      local.tee 10
      local.get 8
      local.get 2
      local.get 3
      call $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE
      br_if 0 (;@1;)
      local.get 1
      local.get 4
      local.get 5
      local.get 10
      i32.load offset=12
      call_indirect (type 1)
      local.set 12
    end
    local.get 12
  )
  (func $_ZN4core3fmt9Formatter12pad_integral12write_prefix17he51f5cf01766db4eE (;23;) (type 10) (param i32 i32 i32 i32 i32) (result i32)
    block ;; label = @1
      local.get 2
      i32.const 1114112
      i32.eq
      br_if 0 (;@1;)
      local.get 0
      local.get 2
      local.get 1
      i32.load offset=16
      call_indirect (type 0)
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1
      return
    end
    block ;; label = @1
      local.get 3
      br_if 0 (;@1;)
      i32.const 0
      return
    end
    local.get 0
    local.get 3
    local.get 4
    local.get 1
    i32.load offset=12
    call_indirect (type 1)
  )
  (func $_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb4e91fd13b3ed913E (;24;) (type 11) (param i32 i32 i32 i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32)
    local.get 1
    local.set 4
    local.get 3
    local.set 5
    block ;; label = @1
      local.get 1
      i32.const 1000
      i32.lt_u
      br_if 0 (;@1;)
      local.get 2
      i32.const -4
      i32.add
      local.set 6
      local.get 3
      local.set 5
      local.get 1
      local.set 7
      loop ;; label = @2
        local.get 6
        local.get 5
        i32.add
        local.tee 8
        i32.const 1
        i32.add
        local.get 7
        local.get 7
        i32.const 10000
        i32.div_u
        local.tee 4
        i32.const 10000
        i32.mul
        i32.sub
        local.tee 9
        i32.const 65535
        i32.and
        i32.const 100
        i32.div_u
        local.tee 10
        i32.const 1
        i32.shl
        local.tee 11
        i32.const 1048729
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        local.get 11
        i32.const 1048728
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        i32.const 3
        i32.add
        local.get 9
        local.get 10
        i32.const 100
        i32.mul
        i32.sub
        i32.const 65535
        i32.and
        i32.const 1
        i32.shl
        local.tee 9
        i32.const 1048729
        i32.add
        i32.load8_u
        i32.store8
        local.get 8
        i32.const 2
        i32.add
        local.get 9
        i32.const 1048728
        i32.add
        i32.load8_u
        i32.store8
        local.get 5
        i32.const -4
        i32.add
        local.set 5
        local.get 7
        i32.const 9999999
        i32.gt_u
        local.set 8
        local.get 4
        local.set 7
        local.get 8
        br_if 0 (;@2;)
      end
    end
    block ;; label = @1
      block ;; label = @2
        local.get 4
        i32.const 9
        i32.gt_u
        br_if 0 (;@2;)
        local.get 4
        local.set 7
        br 1 (;@1;)
      end
      local.get 2
      local.get 5
      i32.add
      i32.const -1
      i32.add
      local.get 4
      local.get 4
      i32.const 65535
      i32.and
      i32.const 100
      i32.div_u
      local.tee 7
      i32.const 100
      i32.mul
      i32.sub
      i32.const 65535
      i32.and
      i32.const 1
      i32.shl
      local.tee 8
      i32.const 1048729
      i32.add
      i32.load8_u
      i32.store8
      local.get 2
      local.get 5
      i32.const -2
      i32.add
      local.tee 5
      i32.add
      local.get 8
      i32.const 1048728
      i32.add
      i32.load8_u
      i32.store8
    end
    block ;; label = @1
      block ;; label = @2
        local.get 1
        i32.eqz
        br_if 0 (;@2;)
        local.get 7
        i32.eqz
        br_if 1 (;@1;)
      end
      local.get 2
      local.get 5
      i32.const -1
      i32.add
      local.tee 5
      i32.add
      local.get 7
      i32.const 1
      i32.shl
      i32.const 30
      i32.and
      i32.const 1048729
      i32.add
      i32.load8_u
      i32.store8
    end
    local.get 0
    local.get 3
    local.get 5
    i32.sub
    i32.store offset=4
    local.get 0
    local.get 2
    local.get 5
    i32.add
    i32.store
  )
  (data $.rodata (;0;) (i32.const 1048576) "\01\00\00\00\00\00\00\00\01\00\00\00\ff\ff\ff\ff\00\00\00\00\ff\ff\ff\ff\ff\ff\ff\ff\00\00\00\00\ff\ff\ff\ff\01\00\00\00\00\00\00\00\01\00\00\00hunter/src/lib.rs\00\00\000\00\10\00\11\00\00\00\cf\00\00\00\1c\00\00\00index out of bounds: the len is  but the index is \00\00T\00\10\00 \00\00\00t\00\10\00\12\00\00\0000010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899")
  (@producers
    (language "Rust" "")
    (processed-by "rustc" "1.90.0 (1159e78c4 2025-09-14)")
  )
  (@custom "target_features" (after data) "\08+\0bbulk-memory+\0fbulk-memory-opt+\16call-indirect-overlong+\0amultivalue+\0fmutable-globals+\13nontrapping-fptoint+\0freference-types+\08sign-ext")
)

"#;
