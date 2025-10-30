// Constants for API configuration
// ID: Konstanta untuk konfigurasi API
// EN: Constants for API configuration

export const API_URL = process.env.API_BASE_URL || 'http://localhost:12000';

export const PAGINATION_DEFAULTS = {
  PAGE_SIZE: 10,
  PAGE_SIZE_OPTIONS: [3, 5, 10, 25, 50]
};

export const ERROR_MESSAGES = {
  NETWORK_ERROR: 'Network error occurred. Please check your connection.',
  SERVER_ERROR: 'Server error occurred. Please try again later.',
  UNAUTHORIZED: 'You are not authorized to perform this action.',
  NOT_FOUND: 'The requested resource was not found.'
};

export const SUCCESS_MESSAGES = {
  CREATE_SUCCESS: 'Data created successfully.',
  UPDATE_SUCCESS: 'Data updated successfully.',
  DELETE_SUCCESS: 'Data deleted successfully.'
};