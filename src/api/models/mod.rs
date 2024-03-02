pub mod update_command_status {
    use crate::models::db::{commands::CommandStatus, common::Id};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct UpdateCommandStatusRequest {
        pub command_id: Id,
        pub status: CommandStatus,
    }
}

pub mod fetch_commands {
    use crate::models::db::commands::Command;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct FetchRecentCommandResponse {
        pub command: Command,
    }
}

pub mod register_device {
    use crate::models::db::common::Id;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RegisterDeviceRequest {
        pub device_name: String,
        pub issuer_id: Id,
        pub user_id: Id,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RegisterDeviceResponse {
        pub device_id: Id,
    }
}
