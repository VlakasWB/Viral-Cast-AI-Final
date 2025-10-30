export function safeAlert(msg: string) {
	if (typeof window !== 'undefined' && typeof window.alert === 'function') {
		window.alert(msg);
	}
}
