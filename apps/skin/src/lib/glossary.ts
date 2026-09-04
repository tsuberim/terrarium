export const GAME_TERMS = [
  {
    term: "Credits",
    def: "What you spend to deploy a creature. One credit equals one glim of in-world energy.",
  },
  {
    term: "Glim",
    def: "The standard unit of life-energy (◆). Creatures spend glims to act; at zero they die.",
  },
  {
    term: "Energy",
    def: "Raw fuel inside a creature, shown in glims in the UI.",
  },
  {
    term: "Cash out",
    def: "Break out of your creature's program loop — its energy returns to your credits.",
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
