use uuid::Uuid;
pub fn generate_id_for(domain: &str) -> String {
    format!("{}-{}", domain, Uuid::new_v4())
}