// Authentication utility functions
// ID: Fungsi utilitas untuk autentikasi
// EN: Authentication utility functions

import { browser } from '$app/environment';

/**
 * Mendapatkan access token dari cookies
 * ID: Mendapatkan token akses dari cookies
 * EN: Get access token from cookies
 */
export function getAuthToken(): string | undefined {
  if (browser) {
    // Mendapatkan token dari cookies di browser
    return document.cookie
      .split('; ')
      .find((row) => row.startsWith('access_token='))
      ?.split('=')[1];
  }
  
  // Di server side, token harus disediakan melalui parameter
  return undefined;
}

/**
 * Menyimpan access token ke cookies
 * ID: Menyimpan token akses ke cookies
 * EN: Save access token to cookies
 */
export function setAuthToken(token: string, expiresIn?: number): void {
  if (!browser) return;
  
  let cookie = `access_token=${token}; path=/; samesite=lax`;
  
  if (expiresIn) {
    const expires = new Date();
    expires.setTime(expires.getTime() + expiresIn * 1000);
    cookie += `; expires=${expires.toUTCString()}`;
  }
  
  document.cookie = cookie;
}

/**
 * Menghapus access token dari cookies
 * ID: Menghapus token akses dari cookies
 * EN: Remove access token from cookies
 */
export function removeAuthToken(): void {
  if (!browser) return;
  
  document.cookie = 'access_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT; samesite=lax';
}

/**
 * Memeriksa apakah user sudah login
 * ID: Memeriksa status login user
 * EN: Check if user is logged in
 */
export function isLoggedIn(): boolean {
  return !!getAuthToken();
}