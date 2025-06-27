use anyhow::{anyhow, bail, Result};
use candid::{Encode, Decode};
use clap::{Parser, Subcommand};
use duct::cmd;
use ic_agent::{Agent, identity::Secp256k1Identity, export::Principal};
use scopeguard;
use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration
};

const SECONDS_BEFORE_DEADLINE: u64 = 10;// vote within 10 seconds of deadline
const THRESHOLD_CANISTER: &str = "6g7za-ziaaa-aaaar-qaqja-cai";
const TURQUOISE: &str = "\x1b[0;36m";
const END_COLOUR: &str = "\x1b[0m";

fn assert_running_from_xtask_dir() -> Result<()> {
    // $CARGO_MANIFEST_DIR is where Cargo found `xtask/Cargo.toml`
    let expected: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize()?;
    // where the user actually launched `cargo xtask`
    let actual: PathBuf = env::current_dir()?.canonicalize()?;
    if expected != actual {
        bail!(
            "xtask must be started from {:?}, but the current directory is {:?}",
            expected,
            actual
        );
    }
    Ok(())
}

fn create_and_use_dfx_identity(name: &str, pem_path: &Path) -> anyhow::Result<()> {
    // check if it already exists
    let list = cmd!("dfx", "identity", "list").read()?;
    // collect identity names (strip trailing " *" on the active one)
    let known: HashSet<&str> = list
        .lines()
        .map(|l| l.trim().trim_end_matches(" *"))
        .filter(|l| !l.is_empty())
        .collect();
    if !known.contains(name) {
        cmd!("dfx", "identity", "import", "--disable-encryption", name, pem_path)
            .stderr_to_stdout()
            .stdout_null()
            .run()?;
    }
    cmd!("dfx", "identity", "use", name).run()?;
    Ok(())
}

// cargo xtask setup-environment
fn setup_environment() -> Result<()> {
    let port = cmd!("dfx", "info", "webserver-port").read()?;
    let url  = format!("http://127.0.0.1:{}", port.trim());
    let original_identity = cmd!("dfx", "identity", "whoami").read().unwrap().trim().to_owned();

    assert_running_from_xtask_dir()?;// relative paths depend on this

    let pem_path = Path::new(&env::current_dir()?).join("alpha_vote_test_identity.pem");
    println!("{}Create and use unencrypted alpha_vote_test_identity (avoid password prompts)...{}", TURQUOISE, END_COLOUR);
    create_and_use_dfx_identity("alpha_vote_test_identity", &pem_path)?;
    let _guard = scopeguard::guard((), |_| {
        // Switch back to original identity after setup_environment
        println!("{}Switching back to using original identity ({})...{}", TURQUOISE, original_identity, END_COLOUR);
        let _ = cmd!("dfx", "identity", "use", &original_identity).run();
    });

    println!("Replica will be booted inside a tmux session to divert noisy console output. You can use 'tmux attach -t replica' if needed, and 'CTRL + b, d' to detach.");
    println!("{}Booting replica...{}", TURQUOISE, END_COLOUR);
    let _ = cmd!("tmux", "kill-session", "-t", "replica").stderr_null().run(); // ignore "no session" error
    cmd!("tmux", "new-session", "-d", "-s", "replica").run()?;
    cmd!(
        "tmux", "send-keys", "-t", "replica",
        "dfx start --clean --background",
        "C-m"
    ).run()?; 
    // give replica a few seconds to boot before installing NNS
    let _ = sleep(Duration::from_secs(5));

    let my_account = cmd!("dfx", "ledger", "account-id").read()?.trim().to_owned();
    println!("{}Installing NNS...{}", TURQUOISE, END_COLOUR);
    cmd!(
        "tmux", "send-keys", "-t", "replica",
        format!("dfx nns install --ledger-accounts {}", my_account),
        "C-m"
    ).run()?;
    // give NNS installation a chance to set up the ledger before proceeding
    let _ = sleep(Duration::from_secs(5));

    println!("{}Deploying with 'initial' implementation...{}", TURQUOISE, END_COLOUR);
    std::fs::copy("./dfx_initial.json", "./dfx.json")?;
    cmd!("dfx", "deploy").run()?;

    let alpha_id  = cmd!("dfx","canister","id","alpha_backend").read()?.trim().to_owned();
    let minor_id  = cmd!("dfx","canister","id","alpha_backend_minor_1").read()?.trim().to_owned();
    println!("{}alpha_backend        : {:?}{}", TURQUOISE, alpha_id, END_COLOUR);
    println!("{}alpha_backend_minor_1: {:?}{}", TURQUOISE, minor_id, END_COLOUR);

    println!("{}Funding canisters for neuron creation...{}", TURQUOISE, END_COLOUR);
    for cid in [&alpha_id, &minor_id] {
        let acc = cmd!("dfx","ledger","account-id","--of-principal",cid).read()?.trim().to_owned();
        cmd!("dfx","ledger","transfer",&acc,"--amount","3.0003","--memo","0").run()?;
    }

    // ic agent avoids the need to serialise and deserialise args and output of dfx canister call
    println!("{}Initialise ic agent...{}", TURQUOISE, END_COLOUR);
    let agent = block_on(async {
        let agent = Agent::builder()
            .with_url(&url)
            .with_identity(Secp256k1Identity::from_pem_file(pem_path)?)
            .build()?;
        agent.fetch_root_key().await?;
        Result::<Agent>::Ok(agent)
    })?;  
    println!("{}Staking and configuring neurons via batch_create_neurons...{}", TURQUOISE, END_COLOUR);
    let alpha_nids = block_on(batch_create_neurons(&agent, &alpha_id, 0, 3))?;
    let minor_nids = block_on(batch_create_neurons(&agent, &minor_id, 0, 3))?;
    println!("{}alpha_backend neurons         : {:?}{}", TURQUOISE, alpha_nids, END_COLOUR);
    println!("{}alpha_backend_minor_1 neurons : {:?}{}", TURQUOISE, minor_nids, END_COLOUR);

    println!("{}Reinstall using 'final' implementation...{}", TURQUOISE, END_COLOUR);
    std::fs::copy("./dfx_final.json", "./dfx.json")?;
    reinstall_final("alpha_backend", &alpha_nids)?;
    reinstall_final("alpha_backend_minor_1", &minor_nids)?;

    println!("{}Set alpha_backend neurons to follow alpha_backend_minor neurons...{}", TURQUOISE, END_COLOUR);
    for topic in [0, 4, 14] {
        for i in 0..3 {
            block_on(follow(&agent, &alpha_id, alpha_nids[i], topic, vec![minor_nids[i]]))?;
        }
    }

    println!("{}Hand over control to the threshold canister...{}", TURQUOISE, END_COLOUR);
    cmd!("dfx","canister","update-settings","--yes",&alpha_id,"--add-controller",THRESHOLD_CANISTER).run()?;
    cmd!("dfx","canister","update-settings","--yes",&minor_id,"--add-controller",THRESHOLD_CANISTER).run()?;
    let my_principal = cmd!("dfx","identity","get-principal").read()?.trim().to_owned();
    cmd!("dfx","canister","update-settings","--yes",&alpha_id,"--remove-controller",&my_principal).run()?;
    cmd!("dfx","canister","update-settings","--yes",&minor_id,"--remove-controller",&my_principal).run()?;

    println!("{}Local environment setup finished.{}", TURQUOISE, END_COLOUR);
    println!("Tidy up the local environment when you're done by using the command 'cargo xtask destroy-environment'.");
    println!("E2E tests can be run against this local environment using the command 'cargo xtask run-e2e-tests'.");
    println!("FYI the same scenarios run as lightweight integration tests (mocked NNS not dependent on 'setup-environment') via 'cargo test'.");

    Ok(())
}

// cargo xtask run-e2e-tests
fn run_e2e_tests() -> Result<()> {

    println!("{}TODO - test sub{}", TURQUOISE, END_COLOUR);

    Ok(())
}

// cargo xtask destroy-environment
fn destroy_environment() -> Result<()> {
    
    println!("{}Stopping replica...{}", TURQUOISE, END_COLOUR);
    let _ = cmd!("dfx", "stop").run();
    
    println!("{}Closing tmux session...{}", TURQUOISE, END_COLOUR);
    let _ = cmd!("tmux", "kill-session", "-t", "replica").stderr_null().run();// ignore "no session" error
    
    let port = cmd!("dfx", "info", "webserver-port").read()?;
    println!("{}Ensuring port {} has been freed...{}", TURQUOISE, port, END_COLOUR);// sometimes 'dfx stop' seems to leave PocketIC bound to the port
    let _ = cmd!("fuser", "-k", "-n", "tcp", &port).run();// attempt to kill anything listening on the configured local replica port

    println!("{}Environment teardown finished.{}", TURQUOISE, END_COLOUR);
    Ok(())
}

// for IC agent calls
fn block_on<F: std::future::Future<Output = T>, T>(fut: F) -> T {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(fut)
}

// called by IC agent
async fn batch_create_neurons(
    agent: &Agent,
    canister_id: &str,
    nonce: u64,
    count: u64
) -> Result<Vec<u64>> {
    let arg = Encode!(&nonce, &count)?;
    let bytes = agent
        .update(&Principal::from_text(canister_id)?, "batch_create_neurons")
        .with_arg(arg)
        .call_and_wait()
        .await?;
    let res: Result<Vec<u64>, String> = Decode!(&bytes, Result<Vec<u64>, String>)?;
    res.map_err(|e| anyhow!("batch_create_neurons failed: {e}"))
}

// called by IC agent
async fn follow(
    agent: &Agent,
    controller_id: &str,
    neuron_id: u64,
    topic: i32,
    followees: Vec<u64>
) -> Result<()> {
    let arg = Encode!(&neuron_id, &topic, &followees)?;
    agent
        .update(&Principal::from_text(controller_id)?, "follow")
        .with_arg(arg)
        .call_and_wait()
        .await?;
    Ok(())
}

fn reinstall_final(canister_name: &str, nids: &[u64]) -> Result<()> {
    let arg = format!(
        "(record {{ alpha_vote_neuron_id = {0}; omega_vote_neuron_id = {1}; \
         omega_reject_neuron_id = {2}; seconds_before_deadline_threshold = {3} }})",
        nids[0], nids[1], nids[2], SECONDS_BEFORE_DEADLINE
    );
    cmd!("dfx","deploy", "--yes",canister_name,"--argument",&arg,"--mode=reinstall").run()?;
    Ok(())
}

#[derive(Parser)]                                                                                                                                                      struct Cli {
    #[command(subcommand)]
    command: Commands,
}                                                                                                                                                                                                                                                                                                                                             #[derive(Subcommand)]                                                                                                                                                  enum Commands {
    SetupEnvironment,
    RunE2eTests,
    DestroyEnvironment
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::SetupEnvironment  => setup_environment()?,
        Commands::RunE2eTests => run_e2e_tests()?,
        Commands::DestroyEnvironment => destroy_environment()?,
    }
    Ok(())
}
