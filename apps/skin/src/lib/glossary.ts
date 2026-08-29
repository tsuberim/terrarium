export const GAME_TERMS = [
  {
    term: "Credits",
    def: "What you spend to deploy a creature into the world.",
  },
  {
    term: "Energy",
    def: "Fuel inside a creature. Actions cost energy; at zero it dies.",
  },
  {
    term: "Deploy",
    def: "Place a new creature with a program. Code cannot be changed later.",
  },
  {
    term: "Solid",
    def: "A wall tile. Creatures can dig through or build more.",
  },
  {
    term: "Corpse",
    def: "Energy left behind when a creature dies. Others can eat it.",
  },
  {
    term: "Empty",
    def: "Open ground. Creatures can move through freely.",
  },
  {
    term: "God view",
    def: "Pan and zoom anywhere. Watch the whole world.",
  },
  {
    term: "Follow",
    def: "Keep the camera locked on one creature as it moves.",
  },
] as const;
