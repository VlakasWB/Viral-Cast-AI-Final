// [ID] Test ThemeToggle: render & toggle.
// [EN] ThemeToggle: render & toggle.
import { render, fireEvent } from '@testing-library/svelte';
import '@testing-library/jest-dom';
import ThemeToggle from './ThemeToggle.svelte';

test('[ID] Tampil & toggle | [EN] Renders & toggles', async () => {
	const { getByRole } = render(ThemeToggle);
	const btn = getByRole('button', { name: /toggle theme/i });
	const before = btn.textContent;
	await fireEvent.click(btn);
	expect(btn.textContent).not.toBe(before);
});
