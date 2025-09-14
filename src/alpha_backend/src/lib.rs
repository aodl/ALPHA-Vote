use candid::{CandidType, Principal};
use ic_cdk::api::canister_balance128;
use ic_cdk::update;
use ic_cdk::api::{call, print, time};
use ic_cdk_macros::{init, post_upgrade};
use ic_cdk_timers::{set_timer, set_timer_interval};
use ic_cdk::spawn;
use ic_nns_common::pb::v1::{NeuronId, ProposalId};
use ic_nns_governance_api::pb::v1::{
    manage_neuron_response, manage_neuron::{Command, RegisterVote, Follow, RefreshVotingPower, NeuronIdOrSubaccount},
    ManageNeuron, ManageNeuronResponse, Vote,
};
use serde::{Deserialize, Serialize};

const GOVERNANCE_CANISTER_ID: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";

#[derive(CandidType, Deserialize, Serialize, Clone, Default)]
struct Config {
    alpha_vote_neuron_id: u64,
    omega_vote_neuron_id: u64,
    omega_reject_neuron_id: u64,
    seconds_before_deadline_threshold: u64
}

#[init]
fn init(c: Config) {
    schedule_hourly_vote_check(&c);
    schedule_daily_reconfirmation(&c);
}

#[post_upgrade]
fn post_upgrade(c: Config) {
    schedule_hourly_vote_check(&c);
    schedule_daily_reconfirmation(&c);
}

fn schedule_hourly_vote_check(c: &Config) {

    let alpha = c.alpha_vote_neuron_id;
    let omega = c.omega_vote_neuron_id;
    let reject = c.omega_reject_neuron_id;
    let seconds_before_deadline = c.seconds_before_deadline_threshold;

    fn run(alpha: u64, omega: u64, reject: u64, seconds_before_deadline: u64) -> impl Fn() + 'static {
        move || {
            spawn(async move {
                if let Err(e) = scan_and_process_open_proposals(alpha, omega, reject, seconds_before_deadline).await {
                    print(format!("Failed to scan proposals: {}", e));
                }
            });
        }
    }

    // Fire once, very soon after install/upgrade (cannot perform inter-canister call during init)
    set_timer(std::time::Duration::from_secs(1), run(alpha, omega, reject, seconds_before_deadline));

    // Then keep running hourly
    set_timer_interval(std::time::Duration::from_secs(60 * 60), run(alpha, omega, reject, seconds_before_deadline));
}

fn schedule_daily_reconfirmation(c: &Config) {

    let alpha = c.alpha_vote_neuron_id;
    let omega = c.omega_vote_neuron_id;
    let reject = c.omega_reject_neuron_id;

    let log_and_refresh = move || {
        print(format!("RefreshVotingPower triggered. [{} {} {}]", alpha, omega, reject));
        for id in [alpha, omega, reject] {
            spawn(refresh_voting_power(id));
        }
    };

    // Fire once, very soon after install/upgrade (cannot perform inter-canister call during init)
    set_timer(std::time::Duration::from_secs(1), log_and_refresh.clone());

    // Repeat every 24h
    set_timer_interval(std::time::Duration::from_secs(60 * 60 * 24), log_and_refresh);
}

async fn refresh_voting_power(neuron_id: u64) {
    let req = ManageNeuron {
        neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::NeuronId(NeuronId { id: neuron_id })),
        command: Some(Command::RefreshVotingPower(RefreshVotingPower {})),
        id: None,
    };

    match call::call::<_, (ManageNeuronResponse,)>(
        Principal::from_text(GOVERNANCE_CANISTER_ID).expect("invalid GOVERNANCE_CANISTER_ID"),
        "manage_neuron",
        (req,),
    ).await {
        Err((code, msg)) => {
            println!(
                "ERROR refresh_voting_power(neuron {}) reject: {:?} - {}",
                neuron_id, code, msg
            );
        }
        Ok((resp,)) => {
            match resp.command {
                Some(manage_neuron_response::Command::Error(e)) => {
                    println!(
                        "ERROR refresh_voting_power(neuron {}) response error: {:?} - {}",
                        neuron_id, e.error_type, e.error_message
                    );
                }
                None => {
                    println!(
                        "ERROR refresh_voting_power(neuron {}) empty response.command",
                        neuron_id
                    );
                }
                _ => {
                    // success, no need to log
                }
            }
        }
    }
}

fn is_controller() -> Result<(), String> {
    if ic_cdk::api::is_controller(&ic_cdk::caller()) {
        Ok(())
    } else {
        Err("You are not a controller".to_string())
    }
}

// used ad-hoc to configure cross-subnet consensus canister-controlled neurons
#[update(guard = "is_controller")]
pub async fn follow(
    neuron_id: u64,
    topic: i32,
    followee_ids: Vec<u64>
) -> Result<ManageNeuronResponse, String> {
    let followees: Vec<NeuronId> =
        followee_ids.into_iter().map(|id| NeuronId { id }).collect();

    let mn = ManageNeuron {
        id: Some(NeuronId { id: neuron_id }),
        command: Some(Command::Follow(Follow { topic, followees })),
        ..Default::default()
    };

    call::call::<_, (ManageNeuronResponse,)>(
        Principal::from_text(GOVERNANCE_CANISTER_ID).unwrap(),
        "manage_neuron",
        (mn,),
    )
    .await
    .map(|(resp,)| resp)
    .map_err(|e| format!("manage_neuron failed: {e:?}"))
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct ListProposalInfo {
    pub limit: u32,
    pub before_proposal: Option<ProposalId>,
    pub exclude_topic: Vec<i32>,
    pub include_reward_status: Vec<i32>,
    pub include_status: Vec<i32>,
    pub include_all_manage_neuron_proposals: Option<bool>,
    pub omit_large_fields: Option<bool>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct Ballot {
    pub vote: i32,
    pub voting_power: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ProposalInfo {
    // Only the fields you needed (others omitted for brevity)
    pub id: Option<ProposalId>,
    pub title: Option<String>,
    pub ballots: ::std::collections::HashMap<u64, Ballot>,
    pub status: Option<i32>,
    pub deadline_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
pub struct ListProposalInfoResponse {
    pub proposal_info: Vec<ProposalInfo>,
}

async fn register_vote(
    neuron_id: u64,
    proposal_id: u64,
    vote: Vote,
) -> Result<ManageNeuronResponse, String> {
    let mn = ManageNeuron {
        id: Some(NeuronId { id: neuron_id }),
        command: Some(Command::RegisterVote(RegisterVote {
            proposal: Some(ProposalId { id: proposal_id }),
            vote: vote as i32,
        })),
        ..Default::default()
    };
    call::call::<_, (ManageNeuronResponse,)>(
        Principal::from_text(GOVERNANCE_CANISTER_ID).unwrap(), 
        "manage_neuron", 
        (mn,)
    )
    .await
    .map(|(resp,)| resp)
    .map_err(|e| format!("manage_neuron failed: {e:?}"))
}

const BATCH_SIZE_LIMIT: u32 = 100;
const REWARD_STATUS_ACCEPT_VOTES: i32 = 1;

pub async fn scan_and_process_open_proposals(alpha: u64, omega: u64, reject: u64, seconds_before_deadline: u64) -> Result<(), String> {

    let mut live_proposals_count: u32 = 0;
    let mut actionable_proposals_count: u32 = 0;
    let mut omega_reject_already_actioned_count: u32 = 0;
    let mut omega_reject_actioned_in_this_run_count: u32 = 0;
    let mut next_proposal_id: u64 = 0;
    let mut next_proposal_due_in: u64 = 0;

    let mut before: Option<ProposalId> = None;
    loop {
        let args = ListProposalInfo {
            limit:                    BATCH_SIZE_LIMIT,
            before_proposal:          before.clone(),
            exclude_topic:            vec![],
            include_reward_status:    vec![REWARD_STATUS_ACCEPT_VOTES],
            omit_large_fields:        Some(true),
            ..Default::default()
        };

        let batch = match list_proposals(args)
            .await
            {
                Ok(r) => r.proposal_info,
                Err(e) => {
                    print(format!("ERROR list_proposals failed: {e}"));
                    break;// give up; caller will re‑run later as part of the normal schedule
                }
            };

        if batch.is_empty() {
            break;// no more data
        }

        for proposal in &batch {
            live_proposals_count += 1;
        
            if let Err(e) = process_one_proposal(
                proposal,
                alpha,
                omega,
                reject,
                seconds_before_deadline,
                &mut actionable_proposals_count,
                &mut omega_reject_already_actioned_count,
                &mut omega_reject_actioned_in_this_run_count,
                &mut next_proposal_id,
                &mut next_proposal_due_in,
            )
            .await
            {
                // Log and continue – nothing stops the rest of the loop
                let pid = proposal
                    .id
                    .as_ref()
                    .map(|p| p.id.to_string())
                    .unwrap_or_else(|| "<unknown-id>".to_string());
                print(format!("[P#{pid}] ERROR processing: {e}"));
            }
        }
        
        if batch.len() < BATCH_SIZE_LIMIT as usize {
            break;// final page was smaller than `limit`
        }
    
        before = batch
            .iter()
            .filter_map(|p| p.id.as_ref())
            .map(|pid| pid.id)
            .min()
            .map(|id| ProposalId { id });
    }

    let cycles = canister_balance128();
    let omega_reject_actioned_count = omega_reject_already_actioned_count + omega_reject_actioned_in_this_run_count;

    let base_message = format!(
        "Cycles: {cycles}, Proposals: {live_proposals_count:>2} live,   \
         of which {actionable_proposals_count:>2} actionable,   \
         of which {omega_reject_actioned_count:>2} actioned,   \
         of which {omega_reject_actioned_in_this_run_count:>2} in this run."
    );

    let full_message = if next_proposal_id != 0 {
        format!("{base_message} P#{next_proposal_id} due in {next_proposal_due_in} seconds.")
    } else {
        base_message
    };

    print(full_message);

    Ok(())
}

async fn process_one_proposal(
    proposal: &ProposalInfo,
    alpha_vote_neuron_id: u64,
    omega_vote_neuron_id: u64,
    omega_reject_neuron_id: u64,
    seconds_before_deadline_threshold: u64,    
    actionable_count: &mut u32,
    omega_reject_already_actioned: &mut u32,
    omega_reject_actioned_this_run: &mut u32,
    next_proposal_id: &mut u64,
    next_proposal_due_in: &mut u64,
) -> Result<(), String> {

    let proposal_id = proposal
        .id
        .as_ref()
        .ok_or("proposal.id missing")?
        .id;

    let deadline_ts_secs = proposal
        .deadline_timestamp_seconds
        .unwrap_or(0);

    let diff_secs = deadline_ts_secs
        .saturating_sub(time() / 1_000_000_000)// ns to seconds
        .saturating_sub(seconds_before_deadline_threshold);
        
    if diff_secs != 0 {
        *next_proposal_id = proposal_id;
        *next_proposal_due_in = diff_secs;
        return Ok(()); // nothing to do yet
    }
    
    *actionable_count += 1;

    let should_trigger_omega_reject = match proposal.ballots.get(&omega_reject_neuron_id) {
        Some(b) => b.vote == 0,// not voted yet
        None => {// ballot not found
            print(format!("[P#{proposal_id}] ERROR scanning ballot: Neuron {omega_reject_neuron_id} not found"));
            true// for safety, assume the worst (not voted) and attempt to trigger a vote
        }
    };
    if should_trigger_omega_reject {        
        if let Err(e) = register_vote(omega_reject_neuron_id, proposal_id, Vote::No).await
        {
            print(format!("[P#{proposal_id}] ERROR auto‑voting for neuron {omega_reject_neuron_id}: {e}"));
        }
        *omega_reject_actioned_this_run += 1;
    } else {
        *omega_reject_already_actioned += 1;
    }

    let vote_enum = match proposal.ballots.get(&alpha_vote_neuron_id) {
        Some(b) if b.vote == 1  => Vote::Yes,
        Some(b) if b.vote == 2 => Vote::No,
        _ => {
            // if alpha vote still hasn't voted, trigger a No vote on the alpha vote neuron
            if let Err(e) = register_vote(alpha_vote_neuron_id, proposal_id, Vote::No).await
            {
                print(format!("[P#{proposal_id}] ERROR auto‑voting for neuron {alpha_vote_neuron_id}: {e}"));
            }
            Vote::No
        }
    };

    let should_trigger_omega_vote = match proposal.ballots.get(&omega_vote_neuron_id) {
        Some(b) => b.vote == 0,// not voted yet
        None => {// ballot not found
            print(format!("[P#{proposal_id}] ERROR scanning ballot: Neuron {omega_vote_neuron_id} not found"));
            true// for safety, assume the worst (not voted) and attempt to trigger a vote
        }
    };
    if should_trigger_omega_vote {
        if let Err(e) = register_vote(omega_vote_neuron_id, proposal_id, vote_enum).await
        {
            print(format!("[P#{proposal_id}] ERROR auto‑voting for neuron {omega_vote_neuron_id}: {e}"));
        }
    }

    Ok(())
}

async fn list_proposals(
    args: ListProposalInfo,
) -> Result<ListProposalInfoResponse, String> {
    let res: Result<(ListProposalInfoResponse,), (i32, String)> = call::call(
        Principal::from_text(GOVERNANCE_CANISTER_ID).unwrap(),
        "list_proposals",
        (args,),
    )
    .await
    .map_err(|(code, msg)| (code as i32, msg));

    match res {
        Ok((resp,)) => Ok(resp),
        Err((code, msg)) => Err(format!("Governance call failed ({}): {}", code, msg)),
    }
}
