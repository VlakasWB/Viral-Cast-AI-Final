import { writable, derived } from 'svelte/store';
import type { Product } from '$lib/types/product';

export interface CartItem {
	product: Product;
	quantity: number;
	unit_price: number;
	line_total: number;
}

export interface CartSummary {
	subtotal: number;
	discount: number;
	tax: number;
	total: number;
	itemCount: number;
}

// Cart items store
export const cartItems = writable<CartItem[]>([]);

// Cart visibility store
export const isCartOpen = writable<boolean>(false);

// Tax rate (9% default)
export const TAX_RATE = 0.09;

// Derived store for cart summary
export const cartSummary = derived(cartItems, ($cartItems) => {
	const subtotal = $cartItems.reduce((sum, item) => sum + item.line_total, 0);
	const discount = 0; // Can be implemented later
	const tax = subtotal * TAX_RATE;
	const total = subtotal - discount + tax;
	const itemCount = $cartItems.reduce((sum, item) => sum + item.quantity, 0);

	return {
		subtotal,
		discount,
		tax,
		total,
		itemCount
	} as CartSummary;
});

// Cart actions
export const cartActions = {
	// Add item to cart
	addItem: (product: Product, quantity: number = 1) => {
		cartItems.update((items) => {
			const existingItemIndex = items.findIndex((item) => item.product.uuid === product.uuid);

			const unit_price = parseFloat(product.price);

			if (existingItemIndex >= 0) {
				// Update existing item
				items[existingItemIndex].quantity += quantity;
				items[existingItemIndex].line_total = items[existingItemIndex].quantity * unit_price;
			} else {
				// Add new item
				items.push({
					product,
					quantity,
					unit_price,
					line_total: quantity * unit_price
				});
			}

			return items;
		});
	},

	// Remove item from cart
	removeItem: (productUuid: string) => {
		cartItems.update((items) => {
			return items.filter((item) => item.product.uuid !== productUuid);
		});
	},

	// Update item quantity
	updateQuantity: (productUuid: string, quantity: number) => {
		if (quantity <= 0) {
			cartActions.removeItem(productUuid);
			return;
		}

		cartItems.update((items) => {
			const itemIndex = items.findIndex((item) => item.product.uuid === productUuid);

			if (itemIndex >= 0) {
				items[itemIndex].quantity = quantity;
				items[itemIndex].line_total = quantity * items[itemIndex].unit_price;
			}

			return [...items];
		});
	},

	// Increase quantity
	increaseQuantity: (productUuid: string) => {
		cartItems.update((items) => {
			const itemIndex = items.findIndex((item) => item.product.uuid === productUuid);

			if (itemIndex >= 0) {
				items[itemIndex].quantity += 1;
				items[itemIndex].line_total = items[itemIndex].quantity * items[itemIndex].unit_price;
			}

			return [...items];
		});
	},

	// Decrease quantity
	decreaseQuantity: (productUuid: string) => {
		cartItems.update((items) => {
			const itemIndex = items.findIndex((item) => item.product.uuid === productUuid);

			if (itemIndex >= 0) {
				if (items[itemIndex].quantity > 1) {
					items[itemIndex].quantity -= 1;
					items[itemIndex].line_total = items[itemIndex].quantity * items[itemIndex].unit_price;
					return [...items];
				} else {
					// Remove item if quantity becomes 0
					return items.filter((item) => item.product.uuid !== productUuid);
				}
			}

			return items;
		});
	},

	// Clear entire cart
	clearCart: () => {
		cartItems.set([]);
	},

	// Toggle cart visibility
	toggleCart: () => {
		isCartOpen.update((open) => !open);
	},

	// Open cart
	openCart: () => {
		isCartOpen.set(true);
	},

	// Close cart
	closeCart: () => {
		isCartOpen.set(false);
	},

	// Get cart item count
	getItemCount: () => {
		let count = 0;
		cartItems.subscribe((items) => {
			count = items.reduce((sum, item) => sum + item.quantity, 0);
		})();
		return count;
	},

	// Check if product is in cart
	isInCart: (productUuid: string): boolean => {
		let inCart = false;
		const unsubscribe = cartItems.subscribe((items) => {
			inCart = items.some((item) => item.product.uuid === productUuid);
		});
		unsubscribe();
		return inCart;
	},

	// Get item quantity in cart
	getItemQuantity: (productUuid: string): number => {
		let quantity = 0;
		const unsubscribe = cartItems.subscribe((items) => {
			const item = items.find((item) => item.product.uuid === productUuid);
			quantity = item ? item.quantity : 0;
		});
		unsubscribe();
		return quantity;
	}
};

// Persist cart to localStorage
if (typeof window !== 'undefined') {
	// Load cart from localStorage on initialization
	const savedCart = localStorage.getItem('shopping-cart');
	if (savedCart) {
		try {
			const parsedCart = JSON.parse(savedCart);
			cartItems.set(parsedCart);
		} catch (error) {
			console.error('Error loading cart from localStorage:', error);
		}
	}

	// Save cart to localStorage whenever it changes
	cartItems.subscribe((items) => {
		localStorage.setItem('shopping-cart', JSON.stringify(items));
	});
}
