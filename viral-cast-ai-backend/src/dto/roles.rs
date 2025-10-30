use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Pagination {
    pub current_page: i64,
    pub total_pages: i64,
    pub next_page: Option<i64>,
    pub prev_page: Option<i64>,
    pub total_available_records: i64,
    pub total_displayed_records: i64,
    pub total_remaining_records: i64,
}

#[derive(Serialize, Deserialize)]
pub struct GetRolesSchema {
    pub uuid: Option<Uuid>,
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, json, to_string};

    #[test]
    fn roles_filter_options_defaults() {
        let opts: FilterOptions = FilterOptions {
            page: None,
            limit: None,
        };
        assert!(opts.page.is_none());
        assert!(opts.limit.is_none());
    }

    #[test]
    fn roles_pagination_defaults_and_update() {
        let mut p = Pagination::default();
        assert_eq!(p.current_page, 0);
        assert_eq!(p.total_pages, 0);
        assert!(p.next_page.is_none());
        assert!(p.prev_page.is_none());
        assert_eq!(p.total_available_records, 0);
        assert_eq!(p.total_displayed_records, 0);
        assert_eq!(p.total_remaining_records, 0);

        // Simulasikan update nilai
        p.current_page = 1;
        p.total_pages = 3;
        p.next_page = Some(2);
        p.prev_page = None;
        p.total_available_records = 30;
        p.total_displayed_records = 10;
        p.total_remaining_records = 20;

        assert_eq!(p.current_page, 1);
        assert_eq!(p.total_pages, 3);
        assert_eq!(p.next_page, Some(2));
        assert_eq!(p.prev_page, None);
        assert_eq!(p.total_available_records, 30);
        assert_eq!(p.total_displayed_records, 10);
        assert_eq!(p.total_remaining_records, 20);
    }

    #[test]
    fn roles_getroles_serde_roundtrip() {
        let payload = GetRolesSchema {
            uuid: None,
            name: Some("Manager".to_string()),
        };
        let s = to_string(&payload).expect("serialize");
        let de: GetRolesSchema = from_str(&s).expect("deserialize");
        assert_eq!(de.name.as_deref(), Some("Manager"));
        assert!(de.uuid.is_none());
    }

    #[test]
    fn roles_param_options_deserialize() {
        let v = json!({"id":"5"}).to_string();
        let p: ParamOptions = from_str(&v).expect("deserialize ParamOptions");
        assert_eq!(p.id, "5");
    }
}
