//! Shared test utilities for ftl-common tests

use async_trait::async_trait;
use ftl_core::deps::{CommandExecutor, CommandOutput};
use std::sync::{Arc, Mutex};

// Simple manual mock implementation for CommandExecutor
// This avoids mockall's issues with async traits containing slice references

type CommandCheckFn = dyn Fn(&str) -> anyhow::Result<()> + Send + Sync;
type CommandExecFn = dyn Fn(&str, &[&str]) -> anyhow::Result<CommandOutput> + Send + Sync;

pub struct MockCommandExecutorMock {
    check_command_exists_fn: Arc<Mutex<Option<Box<CommandCheckFn>>>>,
    execute_fns: Arc<Mutex<Vec<Box<CommandExecFn>>>>,
    execute_call_count: Arc<Mutex<usize>>,
}

impl MockCommandExecutorMock {
    pub fn new() -> Self {
        Self {
            check_command_exists_fn: Arc::new(Mutex::new(None)),
            execute_fns: Arc::new(Mutex::new(Vec::new())),
            execute_call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn expect_check_command_exists(&mut self) -> CheckCommandExistsExpectation {
        CheckCommandExistsExpectation { mock: self }
    }

    pub fn expect_execute(&mut self) -> ExecuteExpectation {
        ExecuteExpectation {
            mock: self,
            matcher: None,
        }
    }
}

pub struct CheckCommandExistsExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
}

impl<'a> CheckCommandExistsExpectation<'a> {
    pub fn with<P>(self, _p: P) -> Self {
        self
    }

    pub fn times(self, _n: usize) -> Self {
        self
    }

    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock
    where
        F: Fn(&str) -> anyhow::Result<()> + Send + Sync + 'static,
    {
        *self.mock.check_command_exists_fn.lock().unwrap() = Some(Box::new(f));
        self.mock
    }
}

pub struct ExecuteExpectation<'a> {
    mock: &'a mut MockCommandExecutorMock,
    matcher: Option<Box<dyn Fn(&str, &[&str]) -> bool + Send + Sync>>,
}

impl<'a> ExecuteExpectation<'a> {
    pub fn withf<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &[&str]) -> bool + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(f));
        self
    }

    pub fn times(self, _n: usize) -> Self {
        self
    }

    pub fn returning<F>(self, f: F) -> &'a mut MockCommandExecutorMock
    where
        F: Fn(&str, &[&str]) -> anyhow::Result<CommandOutput> + Send + Sync + 'static,
    {
        self.mock.execute_fns.lock().unwrap().push(Box::new(f));
        self.mock
    }
}

#[async_trait]
impl CommandExecutor for MockCommandExecutorMock {
    async fn check_command_exists(&self, command: &str) -> anyhow::Result<()> {
        if let Some(ref f) = *self.check_command_exists_fn.lock().unwrap() {
            f(command)
        } else {
            Ok(())
        }
    }

    async fn execute(&self, command: &str, args: &[&str]) -> anyhow::Result<CommandOutput> {
        let mut count = self.execute_call_count.lock().unwrap();
        let index = *count;
        *count += 1;

        let fns = self.execute_fns.lock().unwrap();
        if index < fns.len() {
            fns[index](command, args)
        } else {
            Ok(CommandOutput {
                success: true,
                stdout: vec![],
                stderr: vec![],
            })
        }
    }

    async fn execute_with_stdin(
        &self,
        command: &str,
        args: &[&str],
        _stdin: &str,
    ) -> anyhow::Result<CommandOutput> {
        // For these tests, we don't use stdin, so just delegate to execute
        self.execute(command, args).await
    }
}