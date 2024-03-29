pub mod update_command_status {
    use crate::models::db::{commands::CommandStatus, common::Id};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct UpdateCommandStatusRequest {
        pub command_id: Id,
        pub status: CommandStatus,
    }

    impl UpdateCommandStatusRequest {
        pub fn new(command_id: Id) -> Self {
            Self {
                command_id,
                status: CommandStatus::default(),
            }
        }
    }
}

pub mod fetch_commands {
    use crate::models::db::commands::Command;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct FetchRecentCommandResponse {
        pub command: Command,
    }

    impl FetchRecentCommandResponse {
        pub fn new(command: Command) -> Self {
            FetchRecentCommandResponse { command }
        }
    }
}

pub mod register_device {
    use crate::models::db::common::Id;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RegisterDeviceRequest {
        pub device_name: String,
        pub user_secret: String,
        pub issuer_id: Id,
        pub user_id: Id,
    }

    impl Default for RegisterDeviceRequest {
        fn default() -> Self {
            RegisterDeviceRequest {
                device_name: "testdevicename".to_string(),
                issuer_id: "testissuerid".to_string(),
                user_secret: "testusersecret".to_string(),
                user_id: "testuserid".to_string(),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RegisterDeviceResponse {
        pub device_id: Id,
    }

    impl RegisterDeviceResponse {
        pub fn new(device_id: Id) -> Self {
            RegisterDeviceResponse { device_id }
        }
    }
}
