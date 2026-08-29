export type ExampleProgram = {
  id: string;
  name: string;
  description: string;
  code: string;
};

export const EXAMPLE_PROGRAMS: ExampleProgram[] = [
  {
    id: "idle",
    name: "Idle",
    description: "Sleep forever — cheapest to run",
    code: "loop:\n  sleep\n  jmp loop\n",
  },
  {
    id: "tunnel",
    name: "Tunnel east",
    description: "Move east and dig each tick",
    code: "start:\n  move e\n  dig e\n  sleep\n  jmp start\n",
  },
  {
    id: "wall",
    name: "Wall north",
    description: "Place a wall if blocked to the north",
    code: "sense n\npush 1\n; solid\neq\njz place_it\njmp done\nplace_it:\n  place n\ndone:\n  sleep\n  jmp done\n",
  },
];
