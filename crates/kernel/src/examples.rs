//! Canonical example programs (keep in sync with apps/skin/src/lib/examples.ts).

pub struct ExampleProgram {
    pub id: &'static str,
    pub code: &'static str,
}

pub const ALL: &[ExampleProgram] = &[
    ExampleProgram {
        id: "idle",
        code: "loop:\n  sleep\n  jmp loop\n",
    },
    ExampleProgram {
        id: "tunnel",
        code: "start:\n  move e\n  dig e\n  sleep\n  jmp start\n",
    },
    ExampleProgram {
        id: "wall",
        code: "sense n\npush 1\n; solid\neq\njz place_it\njmp done\nplace_it:\n  place n\ndone:\n  sleep\n  jmp done\n",
    },
];
