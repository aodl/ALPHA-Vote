use candid::{Nat, Principal};
use ic_base_types::PrincipalId;
use ic_cdk::{
    api::call::call,
    print,
};
use ic_cdk_macros::{query, update};
use ic_nns_common::pb::v1::NeuronId;
use ic_nns_governance_api::pb::v1::{
    manage_neuron::{
        self,                   
        claim_or_refresh::{By, MemoAndController},
        configure::Operation,
        Command as NeuronCommand,
        Configure,
        SetVisibility,
    },
    manage_neuron_response::{Command as NeuronResponseCommand},
    ManageNeuron,
    ManageNeuronResponse,
};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
    StableBTreeMap,
};
use icrc_ledger_client_cdk::{CdkRuntime, ICRC1Client};
use icrc_ledger_types::icrc1::{
    account::Account,
    transfer::TransferArg,
};
use sha2::{Digest, Sha256};
use std::cell::RefCell;

const GOVERNANCE_CANISTER_ID: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";
const LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static NEURON_IDS: RefCell<StableBTreeMap<u64, (), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );
}

fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("You are not a controller".to_string())
    }
}

#[update(guard = "is_controller")]
async fn get_canister_icp_balance() -> Result<Nat, String> {
    let ledger_canister_id = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai")
        .map_err(|e| format!(" Invalid Ledger Principal: {:?}", e))?;

    let canister_principal = ic_cdk::id(); // Get the calling canister's principal

    let account = Account {
        owner: canister_principal,
        subaccount: None, // Use None unless using a subaccount
    };

    let (balance,): (Nat,) = call(ledger_canister_id, "icrc1_balance_of", (account,))
        .await
        .map_err(|e| format!("Failed to query balance: {:?}", e))?;

    Ok(balance)
}

#[update(guard = "is_controller")]
fn persist_neuron_id(neuron_id: u64) -> Result<(), String> {
    NEURON_IDS.with(|neuron_ids| {
        neuron_ids.borrow_mut().insert(neuron_id, ())
    });
    Ok(())
}

#[query]
fn get_neuron_ids() -> Vec<u64> {
    NEURON_IDS.with(|neuron_ids| neuron_ids.borrow().iter().map(|(key, _)| key).collect())
}

/// Computes the bytes of the subaccount to which neuron staking transfers are made. This
/// function must be kept in sync with the Nervous System UI equivalent.
/// This code comes from the IC repo:
/// https://github.com/dfinity/ic/blob/master/rs/nervous_system/common/src/ledger.rs#L211
pub fn compute_neuron_staking_subaccount_bytes(controller: Principal, nonce: u64) -> [u8; 32] {
    const DOMAIN: &[u8] = b"neuron-stake";
    const DOMAIN_LENGTH: [u8; 1] = [0x0c];

    let mut hasher = Sha256::new();
    hasher.update(DOMAIN_LENGTH);
    hasher.update(DOMAIN);
    hasher.update(controller.as_slice());
    hasher.update(nonce.to_be_bytes());
    hasher.finalize().into()
}

#[update(guard = "is_controller")]
async fn batch_create_neurons(initial_nonce: u64, count: u64) -> Result<Vec<u64>, String> {
    let mut nonce        = initial_nonce;
    let mut neuron_ids   = Vec::with_capacity(count as usize);

    for _ in 0..count {
        match create_neuron(nonce).await {
            Ok(id) => {
                print(format!("Created neuron {id} with nonce {nonce}"));
                neuron_ids.push(id);
                nonce += 1;
            }
            Err(e) => {
                print(format!("Failed at nonce {nonce}: {e}"));
                return Err(format!("Aborted during creation (nonce {nonce}): {e}"));
            }
        }
    }

    for &id in &neuron_ids {                     
        if let Err(e) = configure_neuron(id).await {
            print(format!("configure_neuron({id}) failed: {e}"));
            return Err(format!("Configuration failed for neuron {id}: {e}"));
        }
    }

    Ok(neuron_ids)
}

#[update(guard = "is_controller")]
async fn create_neuron(nonce: u64) -> Result<u64, String> {
    
    let ledger_canister_id = Principal::from_text(LEDGER_CANISTER_ID)
        .map_err(|e| format!("Invalid ledger canister id: {:?}", e))?;

    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id,
    };

    let transfer_arg = TransferArg {
        to: Account {
            owner: Principal::from_text(GOVERNANCE_CANISTER_ID)
                .map_err(|e| format!("Invalid governance canister id: {:?}", e))?,
            subaccount: Some(compute_neuron_staking_subaccount_bytes(ic_cdk::id(), nonce)),
        },
        amount: Nat::from(100_000_000u64),           // 1 ICP
        fee:   Some(Nat::from(10_000u64)),           // standard fee
        memo:  Some(nonce.into()),
        from_subaccount: None,
        created_at_time: None,
    };

    client.transfer(transfer_arg).await
        .map_err(|(_code, msg)| format!("ICP transfer failed: {}", msg))?
        .map_err(|e| format!("ICP transfer rejected: {:?}", e))?;

    let neuron_request = ManageNeuron {
        id: None,
        neuron_id_or_subaccount: None,
        command: Some(NeuronCommand::ClaimOrRefresh(manage_neuron::ClaimOrRefresh {
            by: Some(By::MemoAndController(MemoAndController {
                controller: Some(PrincipalId::from(ic_cdk::id())), // this canister
                memo: nonce,
            })),
        })),
    };

    let nns_governance_canister_id =
        Principal::from_text(GOVERNANCE_CANISTER_ID).unwrap(); // already validated above

    let (response,): (ManageNeuronResponse,) =
        ic_cdk::call(nns_governance_canister_id, "manage_neuron", (neuron_request,))
            .await
            .map_err(|e| format!("Neuron creation failed: {:?}", e))?;

    let neuron_id = match response.command {
        Some(NeuronResponseCommand::ClaimOrRefresh(r))
            if r.refreshed_neuron_id.is_some() =>
        {
            let id64 = r.refreshed_neuron_id.unwrap().id;
            print(format!("Neuron created with ID: {}", id64));
            // Persist locally if you still want the side‑effect
            persist_neuron_id(id64)?;
            id64
        }
        _ => return Err("Unexpected manage_neuron response".to_string()),
    };

    Ok(neuron_id)
}

#[update(guard = "is_controller")]
async fn configure_neuron(neuron_id: u64) -> Result<(), String> {
    let gov_canister_id = Principal::from_text(GOVERNANCE_CANISTER_ID)
        .map_err(|e| format!("Invalid GOVERNANCE_CANISTER_ID: {}", e))?;

    let increase_delay_request = ManageNeuron {
        id: Some(NeuronId { id: neuron_id }),
        neuron_id_or_subaccount: None,
        command: Some(NeuronCommand::Configure(Configure {
            operation: Some(Operation::IncreaseDissolveDelay(
                manage_neuron::IncreaseDissolveDelay {
                    additional_dissolve_delay_seconds: (8 * 365 * 24 * 60 * 60) as u32,
                },
            )),
        })),
    };

    let response: Result<(ManageNeuronResponse,), _> =
        call(gov_canister_id, "manage_neuron", (increase_delay_request,)).await;

    match response {
        Ok((resp,)) => {
            if let Some(NeuronResponseCommand::Error(e)) = resp.command {
                return Err(format!(
                    "Error from governance canister when increasing delay: {}",
                    e.error_message
                ));
            }
            print(format!("Successfully set max dissolve delay for neuron {neuron_id}"));
        }
        Err(err) => {
            return Err(format!(
                "Failed to call governance (IncreaseDissolveDelay) for neuron {neuron_id}: {err:?}"
            ));
        }
    }

    let auto_stake_request = ManageNeuron {
        id: Some(NeuronId { id: neuron_id }),
        neuron_id_or_subaccount: None,
        command: Some(NeuronCommand::Configure(Configure {
            operation: Some(Operation::ChangeAutoStakeMaturity(
                manage_neuron::ChangeAutoStakeMaturity {
                    requested_setting_for_auto_stake_maturity: true,
                },
            )),
        })),
    };

    let response: Result<(ManageNeuronResponse,), _> =
        call(gov_canister_id, "manage_neuron", (auto_stake_request,)).await;

    match response {
        Ok((resp,)) => {
            if let Some(NeuronResponseCommand::Error(e)) = resp.command {
                return Err(format!(
                    "Error from governance canister when changing auto-stake: {}",
                    e.error_message
                ));
            }
            print(format!("Successfully set auto-stake for neuron {neuron_id}"));
        }
        Err(err) => {
            return Err(format!(
                "Failed to call governance (ChangeAutoStakeMaturity) for neuron {neuron_id}: {err:?}"
            ));
        }
    }

    let visibility_req = ManageNeuron {
        id: Some(NeuronId { id: neuron_id }),
        neuron_id_or_subaccount: None,
        command: Some(NeuronCommand::Configure(Configure {
            operation: Some(Operation::SetVisibility(SetVisibility {
                visibility: Some(2), // 2 = VISIBILITY_PUBLIC
            })),
        })),
    };

    let response: Result<(ManageNeuronResponse,), _> =
        call(gov_canister_id, "manage_neuron", (visibility_req,)).await;

    match response {
        Ok((resp,)) => {
            if let Some(NeuronResponseCommand::Error(e)) = resp.command {
                return Err(format!(
                    "Error setting neuron {neuron_id} visibility to public: {}",
                    e.error_message
                ));
            }
            print(format!("Neuron {neuron_id} is now PUBLIC!"));
        }
        Err(err) => {
            return Err(format!(
                "Failed to call governance (SetVisibility) for neuron {neuron_id}: {err:?}"
            ));
        }
    }

    Ok(())
}
