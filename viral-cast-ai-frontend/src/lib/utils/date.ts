// ID: Utilitas untuk memformat tanggal dan waktu
// EN: Utilities for formatting date and time

export function formatDateTime(timestamp: number): string {
	const date = new Date(timestamp);
	return date.toLocaleString('id-ID', {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit',
		timeZoneName: 'short'
	});
}

export function formatDate(dateString: string): string {
  return new Date(dateString).toLocaleDateString('id-ID', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
}

export function formatTime(dateString: string): string {
  return new Date(dateString).toLocaleTimeString('id-ID', {
    hour: '2-digit',
    minute: '2-digit'
  });
}
