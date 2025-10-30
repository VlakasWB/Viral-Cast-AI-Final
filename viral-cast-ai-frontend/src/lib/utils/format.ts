// Formatting utilities
// ID: Utilitas formatting angka dan mata uang
// EN: Formatting utilities for numbers and currency

/**
 * Format number as currency string
 * @param value - numeric value to format
 * @param currency - ISO 4217 currency code (default: 'IDR')
 * @param locale - BCP 47 locale string (default: 'id-ID')
 */
export function formatCurrency(value: number | null | undefined, currency: string = 'IDR', locale: string = 'id-ID'): string {
  if (value === null || value === undefined || isNaN(Number(value))) return '-';
  try {
    return new Intl.NumberFormat(locale, {
      style: 'currency',
      currency,
      maximumFractionDigits: 2
    }).format(Number(value));
  } catch {
    // Fallback simple formatting
    return `${currency} ${Number(value).toLocaleString(locale)}`;
  }
}

/**
 * Format plain number with grouping
 */
export function formatNumber(value: number | null | undefined, locale: string = 'id-ID'): string {
  if (value === null || value === undefined || isNaN(Number(value))) return '-';
  return new Intl.NumberFormat(locale).format(Number(value));
}

/**
 * Format percentage
 */
export function formatPercent(value: number | null | undefined, locale: string = 'id-ID'): string {
  if (value === null || value === undefined || isNaN(Number(value))) return '-';
  return new Intl.NumberFormat(locale, { style: 'percent', maximumFractionDigits: 2 }).format(Number(value));
}