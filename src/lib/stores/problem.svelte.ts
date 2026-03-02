import type { HintData, ProblemState, ProblemSummary } from "../api/types";

export const problemStore = createProblemStore();

function createProblemStore() {
  let state = $state<ProblemState | null>(null);
  let problems = $state<ProblemSummary[]>([]);
  let feedback = $state<string | null>(null);
  let feedbackType = $state<"correct" | "wrong" | "solved" | "failed" | null>(null);
  let hintData = $state<HintData | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  return {
    get state() {
      return state;
    },
    get problems() {
      return problems;
    },
    get feedback() {
      return feedback;
    },
    get feedbackType() {
      return feedbackType;
    },
    get hintData() {
      return hintData;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    setState(newState: ProblemState) {
      state = newState;
      error = null;
    },
    setProblems(list: ProblemSummary[]) {
      problems = list;
    },
    setFeedback(msg: string | null, type: "correct" | "wrong" | "solved" | "failed" | null = null) {
      feedback = msg;
      feedbackType = type;
    },
    setHint(data: HintData | null) {
      hintData = data;
    },
    setLoading(value: boolean) {
      loading = value;
    },
    setError(msg: string) {
      error = msg;
      loading = false;
    },
    clear() {
      state = null;
      feedback = null;
      feedbackType = null;
      hintData = null;
      error = null;
      loading = false;
    },
    clearFeedback() {
      feedback = null;
      feedbackType = null;
    },
  };
}
