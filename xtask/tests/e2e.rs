use anyhow::Result;
//use tests::skip_neuron_management_proposals;
use duct::cmd;
use ic_agent::{Agent, identity::Secp256k1Identity, export::Principal};
use std::{
    env,
    path::Path
};

// Tests below are divided into:
//   - e2e tests. These only make sense as end-to-end tests because they test how alpha_backend interacts with real NNS functionality.
//   - integration tests. These focus on what alpha_backend does with the information it gets back from the NNS (the NNS can simply be mocked)
// The integration tests 

// TODO establish what the best approach is for setting up shared context for tests

// Another test is does it return any proposals
// Test that behaves as intended when there are no proposals to return
// Test that the neuron is refreshed as intended

#[test]
#[ignore]
fn e2e_skip_neuron_management_proposals() -> Result<()> {

	let pem_path = Path::new(&env::current_dir()?).join("alpha_vote_test_identity.pem"); 
    let port = cmd!("dfx", "info", "webserver-port").read()?;
	let url  = format!("http://127.0.0.1:{}", port.trim());    
	let agent = Agent::builder()
		.with_url(&url)
		.with_identity(Secp256k1Identity::from_pem_file(pem_path)?)
		.build()?;

    let alpha_backend_id = cmd!("dfx", "canister", "id", "alpha_backend").read()?;
    let alpha_backend_principal = Principal::from_text(alpha_backend_id.trim())?;

    //skip_neuron_management_proposals(&agent, &alpha_backend_principal)?;

    // TODO teardown 

    Ok(())
}
