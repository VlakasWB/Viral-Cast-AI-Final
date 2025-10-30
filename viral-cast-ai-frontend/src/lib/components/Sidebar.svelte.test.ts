// [ID] Sidebar buka/tutup terkontrol via prop.
// [EN] Sidebar opens/closes controlled via prop.
import { render, fireEvent } from '@testing-library/svelte';
import '@testing-library/jest-dom';
import Sidebar from './Sidebar.svelte';

test('[ID] Overlay klik menutup | [EN] Overlay click closes', async () => {
	const spy = vi.fn();
	const { getByLabelText } = render(Sidebar, { open: true, onClose: spy });
	await fireEvent.click(getByLabelText('Sidebar').previousElementSibling!); // overlay div
	expect(spy).toHaveBeenCalled();
});
