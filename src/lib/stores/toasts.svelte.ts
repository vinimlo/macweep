export type ToastType = 'success' | 'error' | 'info' | 'warning';

interface Toast {
	id: string;
	type: ToastType;
	message: string;
	duration: number;
}

class ToastStore {
	toasts = $state<Toast[]>([]);
	private timers = new Map<string, ReturnType<typeof setTimeout>>();

	add(type: ToastType, message: string, duration = 4000) {
		const id = crypto.randomUUID();
		this.toasts = [...this.toasts, { id, type, message, duration }];
		this.timers.set(id, setTimeout(() => this.remove(id), duration));
	}

	remove(id: string) {
		const timer = this.timers.get(id);
		if (timer) {
			clearTimeout(timer);
			this.timers.delete(id);
		}
		this.toasts = this.toasts.filter((t) => t.id !== id);
	}

	success(message: string) {
		this.add('success', message);
	}

	error(message: string) {
		this.add('error', message, 6000);
	}

	info(message: string) {
		this.add('info', message);
	}

	warning(message: string) {
		this.add('warning', message, 5000);
	}
}

export const toastStore = new ToastStore();
