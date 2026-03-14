use crate::services::instance::dtos::get_instance_response_dto::GetInstanceResponseDto;

pub mod dtos;

pub struct InstanceService {
    pub instance_id: String,
}

impl InstanceService {
    pub fn new() -> Self {
        let instance_id =
            std::env::var("INSTANCE_ID").unwrap_or_else(|_| "instance_id_not_set".to_string());

        InstanceService { instance_id }
    }

    pub fn get_instance(&self) -> GetInstanceResponseDto {
        GetInstanceResponseDto {
            instance_id: self.instance_id.clone(),
        }
    }
}
