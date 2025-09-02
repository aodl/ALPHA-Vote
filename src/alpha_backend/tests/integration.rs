use alpha_backend::Config;
use anyhow::Result;
use candid::encode_one;
use ic_agent::{Agent, identity::Secp256k1Identity, export::Principal};
use ic_nns_governance::pb::v1::ProposalInfo;
use pocket_ic::PocketIc;
use tests::process_proposals;
use std::{
    fs,
    env,
    path::Path
};

// Some integration tests can also be run as e2e tests using xtask, dfx, and an instaleld NNS (rather than mocking NNS infrastructure). 
// These types of tests defer to the shared TODO crate, for test harness flexibility (they're agnostic to how the context is setup up).
//
// Other integration tests depend on features such as fast forwarding (which isn't yet support via dfx).
// These types of tests are self contained in this test crate, and cannot yet be called as e2e tests.
//
// See notes in TODO for more info about running e2e tests.

const GOVERNANCE_CANISTER_ID: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";

fn setup_replica() -> Result<(Agent, PocketIc, Principal)> {
    
    // path to the crate that contains alpha_backend
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // ../../target/wasm32-unknown-unknown/release/alpha_backend.wasm
    let backend_wasm = crate_root
        .join("..")
        .join("..")
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("alpha_backend.wasm");

    let args = Config {
        alpha_vote_neuron_id: 0,
        omega_vote_neuron_id: 0,
        omega_reject_neuron_id: 0,
        seconds_before_deadline_threshold: 172_800,// 2 days 
    };
    let encoded_args = encode_one(args)?;

    let pic = PocketIc::new();
    let backend_canister = pic.create_canister();
    pic.add_cycles(backend_canister, 2_000_000_000_000);
    let wasm = fs::read(backend_wasm).expect("Wasm file not found, run 'dfx build'.");
    pic.install_canister(backend_canister, wasm, encoded_args, None);
    
    let url = pic.auto_progress().to_string(); 
    //pic.url().expect("PocketIC has no URL yet").to_string();
    let pem_path = Path::new(&env::current_dir()?).join("../../xtask/alpha_vote_test_identity.pem");
    let agent = Agent::builder()
        .with_url(url)
        .with_identity(Secp256k1Identity::from_pem_file(pem_path)?)
        .build()?;
    
    Ok((agent, pic, backend_canister))
}

fn install_mock_nns(pic: &PocketIc, encoded_list_proposals_result: Vec<u8>) -> Result<()> {

    let gov_id = Principal::from_text(GOVERNANCE_CANISTER_ID).unwrap();
    pic.create_canister_with_id(gov_id, None);

    let wasm = ic_universal_canister::wasm();
    pic.install_canister(gov_id, wasm, vec![], None).unwrap();

    let uc = UniversalCanister::from_existing(pic, gov_id);
    uc.update(
        wasm()
            // only answer list_proposals
            .msg_method_name()
            .push_bytes(b"list_proposals")
            .trap_if_ne()
            // always respond with the specified result
            .push_bytes(&encoded_list_proposals_result)
            .reply_data_append()
            .reply()
            .build(),
        (),
    )
    .unwrap();

    Ok(())
}

#[test]
fn process_0_proposals() -> Result<()> {

    let (agent, pic, backend_canister) = setup_replica()?;
    
    let mocked_proposals_result = Encode!(&vec![
        ProposalInfo { id: Some(1), ..Default::default() }
    ])?;

    install_mock_nns(pic, mocked_proposals_result);

    process_proposals(&agent, &backend_canister, 0, 0)?;

    Ok(())
}

#[test]
fn process_1000_proposals() -> Result<()> {

    let (agent, backend_canister) = setup_replica()?;

    // TODO mock NNS with respect to providing actionable and non-actionable proposals when queried

    process_proposals(&agent, &backend_canister, 1000, 0)?;

    // fast-forward time
    process_proposals(&agent, &backend_canister, 500, 500)?;

    // fast-forward time
    process_proposals(&agent, &backend_canister, 0, 1000)?;

    // fast-forward time
    process_proposals(&agent, &backend_canister, 0, 500)?;

    // fast-forward time
    process_proposals(&agent, &backend_canister, 0, 0)?;

    Ok(())
}

#[test]
fn periodic_reconfirmation() -> Result<()> {

    let (agent, backend_canister) = setup_replica()?;

    // assert and count calls to the reconfirmation stuff (maybe by seeing if follow was called)

    Ok(())
}
