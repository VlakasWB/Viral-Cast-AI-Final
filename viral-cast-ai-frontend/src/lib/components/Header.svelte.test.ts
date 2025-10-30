// [ID] Header memanggil onHamburger saat klik.
// [EN] Header calls onHamburger when clicked.
import { render, fireEvent } from '@testing-library/svelte';
import '@testing-library/jest-dom';
import Header from './Header.svelte';

test('[ID] Klik hamburger | [EN] Hamburger click', async () => {
	const spy = vi.fn();
	const { getByRole } = render(Header, { onHamburger: spy });
	await fireEvent.click(getByRole('button', { name: /open sidebar/i }));
	expect(spy).toHaveBeenCalled();
});
