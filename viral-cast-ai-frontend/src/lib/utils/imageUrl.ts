// Utility function to build complete image URLs
export function buildImageUrl(
	imageUrl: string | null,
	baseUrl: string = 'http://localhost:12000'
): string | null {
	if (!imageUrl) return null;

	// If it's already a complete URL, return as is
	if (imageUrl.startsWith('http://') || imageUrl.startsWith('https://')) {
		return imageUrl;
	}

	// If it's a relative path, combine with base URL
	return `${baseUrl}${imageUrl}`;
}
