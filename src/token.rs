use uuid::Uuid;

pub fn create_token() -> String { 
    Uuid::new_v4().to_string()
}