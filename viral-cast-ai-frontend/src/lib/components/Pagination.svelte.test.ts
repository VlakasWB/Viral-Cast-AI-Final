import { render, fireEvent } from '@testing-library/svelte';
import '@testing-library/jest-dom';
import Pagination from './Pagination.svelte';

test('[ID] Next memanggil onChange | [EN] Next triggers onChange', async () => {
	const spy = vi.fn();
	const { getByText } = render(Pagination, { page: 1, pageCount: 3, onChange: spy });
	await fireEvent.click(getByText('Next'));
	expect(spy).toHaveBeenCalledWith(2);
});
