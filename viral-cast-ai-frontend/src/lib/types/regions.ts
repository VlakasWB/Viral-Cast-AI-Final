export interface PaginationInfo {
	total: number;
	limit: number;
	offset: number;
	has_next: boolean;
}

export interface RegionsListResponse<T> {
	success: boolean;
	message: string;
	data: T[];
	pagination: PaginationInfo;
}

export interface SingleRegionResponse<T> {
	success: boolean;
	message: string;
	data: T;
}

export interface Province {
	uuid: string;
	code: string;
	name: string;
}

export interface Regency {
	uuid: string;
	code: string;
	name: string;
	province_uuid: string;
	province_name: string;
	// Optional raw code from backend when available
	province_code?: string;
}

export interface District {
	uuid: string;
	code: string;
	name: string;
	// The documentation example uses "district_uuid" pointing to regency; assume it's a typo and use regency_uuid
	regency_uuid?: string;
	regency_name?: string;
	province_uuid: string;
	province_name: string;
	// Optional raw codes from backend when available
	regency_code?: string;
	province_code?: string;
}

export interface Village {
	uuid: string;
	code: string;
	name: string;
	district_uuid: string;
	district_name: string;
	regency_uuid: string;
	regency_name: string;
	province_uuid: string;
	province_name: string;
	// Optional raw codes from backend when available
	district_code?: string;
	regency_code?: string;
	province_code?: string;
}
