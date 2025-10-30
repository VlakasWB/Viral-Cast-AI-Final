export interface UOM {
	uuid: string;
	code: string;
	name: string;
	created_at: number;
	updated_at: number;
}

export interface CreateUOMRequest {
	name: string;
	code: string;
	description?: string;
}

export interface UpdateUOMRequest {
	name?: string;
	code?: string;
	description?: string;
}

export interface PaginationMeta {
	total: number;
	last_page: number;
	current_page: number;
	per_page: number;
	from: number;
	to: number;
}

export interface UOMResponse {
	code: number;
	status: string;
	message: string;
	data: UOM[];
	meta: PaginationMeta;
	errors: Record<string, any>;
}

export interface SingleUOMResponse {
	code: number;
	status: string;
	message: string;
	data: UOM;
	errors: Record<string, any>;
}

export interface DeleteUOMResponse {
	code: number;
	status: string;
	message: string;
	data: Record<string, any>;
	errors: Record<string, any>;
}
