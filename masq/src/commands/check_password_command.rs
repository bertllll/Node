// Copyright (c) 2019-2020, MASQ (https://masq.ai) and/or its affiliates. All rights reserved.

use clap::{App, SubCommand, Arg};
use crate::commands::commands_common::{Command, CommandError, transaction};
use crate::command_context::CommandContext;
use masq_lib::messages::{UiCheckPasswordRequest, UiCheckPasswordResponse};

#[derive(Debug)]
pub struct CheckPasswordCommand {
    db_password_opt: Option<String>,
}

pub fn check_password_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("check-password")
        .about("Checks whether the supplied db-password (if any) is the correct password for the Node's database")
        .arg(Arg::with_name ("db-password")
            .help ("Password to check--leave it out if you think the database doesn't have a password yet")
            .index (1)
            .required (false)
            .case_insensitive(false)
        )
}

impl Command for CheckPasswordCommand {
    fn execute(&self, context: &mut dyn CommandContext) -> Result<(), CommandError> {
        let input = UiCheckPasswordRequest {
            db_password_opt: self.db_password_opt.clone(),
        };
        let msg: UiCheckPasswordResponse = transaction (input, context, 1000)?;
        writeln!(context.stdout(), "{}", if msg.matches {
            "Password is correct"
        }
        else {
            "Password is incorrect"
        }).expect("writeln! failed");
        Ok(())
    }
}

impl CheckPasswordCommand {
    pub fn new(pieces: Vec<String>) -> Result<Self, String> {
        let matches = match check_password_subcommand().get_matches_from_safe(pieces) {
            Ok(matches) => matches,
            Err(e) => return Err(format!("{}", e)),
        };
        Ok(Self {
            db_password_opt: matches
                .value_of("db-password")
                .map (|r| r.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_context::{ContextError};
    use crate::command_factory::{CommandFactory, CommandFactoryReal};
    use crate::test_utils::mocks::CommandContextMock;
    use masq_lib::messages::{ToMessageBody, UiCheckPasswordRequest, UiCheckPasswordResponse};
    use std::sync::{Arc, Mutex};
    use crate::commands::commands_common::{CommandError, Command};

    #[test]
    fn testing_command_factory_here() {
        let factory = CommandFactoryReal::new();
        let mut context = CommandContextMock::new()
            .transact_result(Ok(UiCheckPasswordResponse {
                matches: true
            }.tmb(0)));
        let subject = factory
            .make(vec![
                "check-password".to_string(),
                "bonkers".to_string(),
            ])
            .unwrap();

        let result = subject.execute(&mut context);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn check_password_command_with_a_password_right() {
        let transact_params_arc = Arc::new(Mutex::new(vec![]));
        let mut context = CommandContextMock::new()
            .transact_params(&transact_params_arc)
            .transact_result(Ok(UiCheckPasswordResponse {
                matches: true
            }.tmb(0)));
        let stdout_arc = context.stdout_arc();
        let stderr_arc = context.stderr_arc();
        let factory = CommandFactoryReal::new();
        let subject = factory
            .make(vec![
                "check-password".to_string(),
                "bonkers".to_string(),
            ])
            .unwrap();

        let result = subject.execute(&mut context);

        assert_eq!(result, Ok(()));
        assert_eq!(stdout_arc.lock().unwrap().get_string(), "Password is correct\n");
        assert_eq!(stderr_arc.lock().unwrap().get_string(), String::new());
        let transact_params = transact_params_arc.lock().unwrap();
        assert_eq!(
            *transact_params,
            vec![(
                UiCheckPasswordRequest {
                    db_password_opt: Some ("bonkers".to_string()),
                }
                .tmb(0),
                1000
            )]
        )
    }

    #[test]
    fn check_password_command_with_no_password_wrong() {
        let transact_params_arc = Arc::new(Mutex::new(vec![]));
        let mut context = CommandContextMock::new()
            .transact_params(&transact_params_arc)
            .transact_result(Ok(UiCheckPasswordResponse {
                matches: false
            }.tmb(0)));
        let stdout_arc = context.stdout_arc();
        let stderr_arc = context.stderr_arc();
        let factory = CommandFactoryReal::new();
        let subject = factory
            .make(vec![
                "check-password".to_string(),
            ])
            .unwrap();

        let result = subject.execute(&mut context);

        assert_eq!(result, Ok(()));
        assert_eq!(stdout_arc.lock().unwrap().get_string(), "Password is incorrect\n");
        assert_eq!(stderr_arc.lock().unwrap().get_string(), String::new());
        let transact_params = transact_params_arc.lock().unwrap();
        assert_eq!(
            *transact_params,
            vec![(
                UiCheckPasswordRequest {
                    db_password_opt: None,
                }
                .tmb(0),
                1000
            )]
        )
    }

    #[test]
    fn check_password_command_handles_send_failure() {
        let mut context = CommandContextMock::new()
            .transact_result (Err (ContextError::ConnectionDropped("tummyache".to_string())));
        let subject = CheckPasswordCommand::new(vec![
            "check-password".to_string(),
            "bonkers".to_string(),
        ])
        .unwrap();

        let result = subject.execute(&mut context);

        assert_eq!(
            result,
            Err(CommandError::ConnectionProblem("tummyache".to_string()))
        )
    }
}