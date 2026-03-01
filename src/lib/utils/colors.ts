import type { Severity } from "../api/types";

export function severityColor(severity: Severity): number {
  switch (severity) {
    case "Good":
      return 0x4caf50;
    case "Inaccuracy":
      return 0xffc107;
    case "Mistake":
      return 0xff9800;
    case "Blunder":
      return 0xf44336;
  }
}

export function severityLabel(severity: Severity): string {
  switch (severity) {
    case "Good":
      return "Good";
    case "Inaccuracy":
      return "Inaccuracy";
    case "Mistake":
      return "Mistake";
    case "Blunder":
      return "Blunder";
  }
}
