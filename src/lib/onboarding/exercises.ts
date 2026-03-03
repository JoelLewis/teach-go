export type TutorialExercise = {
  id: string;
  title: string;
  instruction: string;
  boardSize: number;
  setupBlack: [number, number][];
  setupWhite: [number, number][];
  playerColor: "black" | "white";
  correctMove: [number, number];
  successMessage: string;
};

export const tutorialExercises: TutorialExercise[] = [
  {
    id: "capture",
    title: "Capturing Stones",
    instruction:
      "The white stone has only one liberty left. Place a black stone to capture it!",
    boardSize: 5,
    setupBlack: [
      [1, 2],
      [2, 1],
      [2, 3],
    ],
    setupWhite: [[2, 2]],
    playerColor: "black",
    correctMove: [3, 2],
    successMessage:
      "You captured the white stone by filling its last liberty.",
  },
  {
    id: "ko",
    title: "The Ko Rule",
    instruction:
      "Black can capture the white stone at the marked point. Try it!",
    boardSize: 5,
    setupBlack: [
      [1, 2],
      [2, 1],
      [3, 2],
      [2, 3],
    ],
    setupWhite: [
      [1, 3],
      [2, 2],
      [3, 3],
    ],
    playerColor: "black",
    correctMove: [2, 4],
    successMessage:
      "After capturing, White cannot immediately take back — that's the ko rule. Players must play elsewhere first.",
  },
  {
    id: "territory",
    title: "Counting Territory",
    instruction:
      "Black has surrounded territory in the corner. Tap the key intersection to secure it!",
    boardSize: 5,
    setupBlack: [
      [0, 2],
      [1, 2],
      [2, 0],
      [2, 1],
      [2, 2],
    ],
    setupWhite: [
      [3, 0],
      [3, 1],
      [3, 2],
      [3, 3],
    ],
    playerColor: "black",
    correctMove: [0, 0],
    successMessage:
      "Black's corner is now secured. The empty points surrounded by your stones are your territory.",
  },
  {
    id: "life_death",
    title: "Life and Death",
    instruction:
      "White's group needs two eyes to live. Find the vital point to kill it!",
    boardSize: 5,
    setupBlack: [
      [0, 0],
      [0, 1],
      [0, 3],
      [0, 4],
      [1, 0],
      [1, 4],
      [2, 0],
      [2, 1],
      [2, 2],
      [2, 3],
      [2, 4],
    ],
    setupWhite: [
      [0, 2],
      [1, 1],
      [1, 2],
      [1, 3],
    ],
    playerColor: "black",
    correctMove: [0, 2],
    successMessage:
      "By playing at the vital point, White cannot make two eyes. The group is dead!",
  },
];
