use anyhow::Result;
use ic_agent::{Agent, export::Principal};

// These tests can be called either as:
//   - lightweight integration tests using PocketIC and mocked NNS infrastructure. Use `cargo test`.
//   - or, heavy end-to-end tests using xtask and dfx to setup, install, and initialise local environment against which the tests run. Use `xtask TODO`
//
// Sharing the definition of test cases like this means that if an integration test is failing... TODO

pub fn process_proposals(agent: &Agent, principal: &Principal, actionable_count: u32, non_actionable_count: u32) -> Result<()> {

    // context set up by caller (including preparing the proposals, given that setup is different for integration vs e2e)
    // trigger processing
    // check logs are what we expect

    Ok(())
}
