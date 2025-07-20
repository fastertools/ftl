pub mod add;
pub mod auth;
pub mod build;
pub mod deploy;
pub mod init;
pub mod login;
pub mod logout;
pub mod publish;
pub mod registry;
pub mod setup;
pub mod test;
pub mod up;
pub mod update;

#[cfg(test)]
mod add_tests;
#[cfg(test)]
mod auth_tests;
#[cfg(test)]
mod build_tests;
#[cfg(test)]
mod deploy_tests;
#[cfg(test)]
mod init_tests;
#[cfg(test)]
mod login_tests;
#[cfg(test)]
mod logout_tests;
#[cfg(test)]
mod publish_tests;
#[cfg(test)]
mod registry_tests;
#[cfg(test)]
mod setup_tests;
#[cfg(test)]
mod test_tests;
#[cfg(test)]
mod up_tests;
#[cfg(test)]
mod update_tests;
