use crate::services::instance::dtos::get_instance_response_dto::GetInstanceResponseDto;

pub mod dtos;

pub struct InstanceService {
    pub instance_id: String,
}

impl InstanceService {
    pub fn new() -> Self {
        let instance_id = std::env::var("INSTANCE_ID").expect("INSTANCE_ID must be set");

        InstanceService { instance_id }
    }

    pub fn get_instance(&self) -> GetInstanceResponseDto {
        GetInstanceResponseDto {
            instance_id: self.instance_id.clone(),
        }
    }
}
