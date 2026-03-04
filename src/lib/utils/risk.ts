import type { RiskLevel } from '$lib/tauri/types';

export function riskColor(level: RiskLevel): string {
	const colors: Record<RiskLevel, string> = {
		Zero: 'var(--risk-zero)',
		Low: 'var(--risk-low)',
		Medium: 'var(--risk-medium)',
		High: 'var(--risk-high)'
	};
	return colors[level];
}

export function riskBgColor(level: RiskLevel): string {
	const colors: Record<RiskLevel, string> = {
		Zero: 'var(--risk-zero-dim)',
		Low: 'var(--risk-low-dim)',
		Medium: 'var(--risk-medium-dim)',
		High: 'var(--risk-high-dim)'
	};
	return colors[level];
}

export function riskLabel(level: RiskLevel): string {
	const labels: Record<RiskLevel, string> = {
		Zero: 'Safe',
		Low: 'Low Risk',
		Medium: 'Medium Risk',
		High: 'High Risk'
	};
	return labels[level];
}

export function riskIcon(level: RiskLevel): string {
	const icons: Record<RiskLevel, string> = {
		Zero: '\u25CF',
		Low: '\u25CF',
		Medium: '\u25B2',
		High: '\u25C6'
	};
	return icons[level];
}

export function riskOrder(level: RiskLevel): number {
	const order: Record<RiskLevel, number> = { Zero: 0, Low: 1, Medium: 2, High: 3 };
	return order[level];
}
