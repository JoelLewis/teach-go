export type TutorialExercise = {
  id: string;
  title: string;
  instruction: string;
  boardSize: number;
  setupBlack: [number, number][];
  setupWhite: [number, number][];
  playerColor: "black" | "white";
  correctMove: [number, number];
  captures: [number, number][];
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
    captures: [[2, 2]],
    successMessage:
      "You captured the white stone by filling its last liberty.",
  },
  {
    id: "ko",
    title: "The Ko Rule",
    instruction:
      "White's stone at the marked point has only one liberty. Place a black stone to capture it!",
    boardSize: 5,
    // Board:
    //      0   1   2   3   4
    // 0    .   .   .   .   .
    // 1    .   B   W   .   .
    // 2    B   W   .   W   .
    // 3    .   B   W   .   .
    // 4    .   .   .   .   .
    setupBlack: [
      [1, 1],
      [2, 0],
      [3, 1],
    ],
    setupWhite: [
      [1, 2],
      [2, 1],
      [2, 3],
      [3, 2],
    ],
    playerColor: "black",
    correctMove: [2, 2],
    captures: [[2, 1]],
    successMessage:
      "You captured the white stone! Now White cannot immediately recapture at that point — that's the Ko rule. White must play elsewhere first.",
  },
  {
    id: "territory",
    title: "Securing Territory",
    instruction:
      "Black's wall has a gap at the marked point. Close it before White can invade!",
    boardSize: 5,
    // Board:
    //      0   1   2   3   4
    // 0    .   .   B   .   .
    // 1    .   .   B   .   .
    // 2    B   .   B   .   .
    // 3    B   B   B   .   .
    // 4    W   W   W   .   .
    setupBlack: [
      [0, 2],
      [1, 2],
      [2, 0],
      [2, 2],
      [3, 0],
      [3, 1],
      [3, 2],
    ],
    setupWhite: [
      [4, 0],
      [4, 1],
      [4, 2],
    ],
    playerColor: "black",
    correctMove: [2, 1],
    captures: [],
    successMessage:
      "The wall is sealed! The 4 empty points in the corner are now Black's territory — fully enclosed and safe from invasion.",
  },
  {
    id: "life_death",
    title: "Life and Death",
    instruction:
      "White's group can form two eyes if it plays the center. Play there first to prevent it!",
    boardSize: 7,
    // Board:
    //      0   1   2   3   4   5   6
    // 0    .   W   .   .   .   W   .
    // 1    B   W   W   W   W   W   B
    // 2    .   B   B   B   B   B   .
    setupBlack: [
      [1, 0],
      [1, 6],
      [2, 1],
      [2, 2],
      [2, 3],
      [2, 4],
      [2, 5],
    ],
    setupWhite: [
      [0, 1],
      [0, 5],
      [1, 1],
      [1, 2],
      [1, 3],
      [1, 4],
      [1, 5],
    ],
    playerColor: "black",
    correctMove: [0, 3],
    captures: [],
    successMessage:
      "By playing the vital point, White can no longer divide the space into two eyes. The group is dead!",
  },
];
