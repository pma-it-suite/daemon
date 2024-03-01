    use crate::models::HandlerError;
    use jfs;
    use lazy_static::lazy_static;
    use log::{info};
    use std::collections::HashMap;
    use std::sync::Mutex;

    fn get_file_path() -> String {
        "localstore.json".to_string()
    }

    lazy_static! {
        static ref HANDLE: Mutex<Result<jfs::Store, HandlerError>> = Mutex::new(get_handle());
    }

    fn get_handle() -> Result<jfs::Store, HandlerError> {
        let db = jfs::Store::new_with_cfg(
            get_file_path(),
            jfs::Config {
                single: true,
                pretty: true,
                ..Default::default()
            },
        )?;
        init_db(&db)?;
        Ok(db)
    }

    fn init_db(db: &jfs::Store) -> Result<(), HandlerError> {
        let key = "user_id";
        let resp = query_internal(db, key);
        if resp.is_err() || resp?.is_none() {
            let user_id = "ee9470de-54a4-419c-b34a-ba2fa18731d8";
            db.save_with_id(&user_id.to_string(), key)?;
            info!("user_id set to default: {}", user_id);
        }
        Ok(())
    }

    pub fn write_single(data: &String, key: &str) -> Result<(), HandlerError> {
        info!("writing data for key: {}", key);
        let binding = HANDLE.lock().unwrap();
        let handle = binding.as_ref().unwrap();
        handle.save_with_id(data, key)?;
        Ok(())
    }

    pub fn write_data(data: HashMap<String, String>) -> Result<(), HandlerError> {
        data.keys().for_each(|key| {
            write_single(&data[key], key).unwrap();
        });
        Ok(())
    }

    fn query_internal(db: &jfs::Store, key: &str) -> Result<Option<String>, HandlerError> {
        let data = db.get(key)?;
        Ok(data)
    }

    pub fn query_data(key: &str) -> Result<Option<String>, HandlerError> {
        info!("querying data for key: {}", key);
        let binding = HANDLE.lock().unwrap();
        let handle = binding.as_ref().unwrap();
        query_internal(handle, key)
    }