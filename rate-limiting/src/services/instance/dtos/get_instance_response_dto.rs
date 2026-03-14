use serde::Serialize;

#[derive(Serialize)]
pub struct GetInstanceResponseDto {
    pub instance_id: String,
}
