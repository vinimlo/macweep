import type { RiskLevel } from "$lib/tauri/types";

export function riskColor(level: RiskLevel): string {
  const colors: Record<RiskLevel, string> = {
    Zero: "var(--risk-zero)",
    Low: "var(--risk-low)",
    Medium: "var(--risk-medium)",
    High: "var(--risk-high)",
  };
  return colors[level];
}

export function riskLabel(level: RiskLevel): string {
  const labels: Record<RiskLevel, string> = {
    Zero: "Safe",
    Low: "Low Risk",
    Medium: "Medium Risk",
    High: "High Risk",
  };
  return labels[level];
}

export function riskOrder(level: RiskLevel): number {
  const order: Record<RiskLevel, number> = {
    Zero: 0,
    Low: 1,
    Medium: 2,
    High: 3,
  };
  return order[level];
}
