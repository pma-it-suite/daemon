    use log::info;

    use crate::models::db::commands::{Command, CommandNames};
    use crate::models::HandlerError;

    pub async fn handoff_command_to_executor(
        command: &Command,
    ) -> Result<Option<String>, HandlerError> {
        info!("handing off command to executor: {:?}", &command);
        match &command.name {
            CommandNames::Test => {
                // TODO @felipearce: add test command here
                Ok(Some("test".to_string()))
            }
            CommandNames::ShellCmd => {
                // execute args in the shell
                let args = match command.args.as_ref() {
                    Some(args) => args,
                    None => {
                        return Err(HandlerError::ParseError(
                            "no args found for shell cmd".to_string(),
                        ));
                    }
                };

                let output_result = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(args)
                    .output();

                match output_result {
                    Ok(output) => {
                        let output_str = String::from_utf8(output.stdout).unwrap();
                        Ok(Some(output_str))
                    }
                    Err(e) => Err(HandlerError::CmdError(e.to_string())),
                }
            }
            _ => {
                // TODO @felipearce: add more commands here
                Ok(None)
            }
        }
    }