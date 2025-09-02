#!/usr/bin/env bash
# create_neurons_simple.sh
# Source it (  source ./create_neurons_simple.sh  ) to keep the variables.

set -euo pipefail

IC_URL=http://127.0.0.1:8080/
PEM=~/.config/dfx/identity/ident-1/identity.pem
AMT=20.5

stake () {
  local name=$1 amount=$2 pem=$3 raw id

  raw=$(quill neuron-stake --amount "$amount" --name "$name" --pem-file "$pem" \
        | IC_URL="$IC_URL" quill --insecure-local-dev-mode send -y -)

  # example line:  Successfully staked ICP in neuron 17821076663347300019
  id=$(grep -oE 'neuron[[:space:]]+[0-9]+' <<<"$raw" | \
       tail -n1 | awk '{print $2}')

  [[ -n $id ]] || { echo "Could not find neuron id for $name" >&2; return 1; }
  printf '%s\n' "$id"
}

AV_NEURON_ID=$(stake AV  "$AMT" "$PEM")
OV_NEURON_ID=$(stake OV  "$AMT" "$PEM")
OR_NEURON_ID=$(stake OR  "$AMT" "$PEM")
MOCK_D_QUORUM_NEURON_ID=$(stake Mock  "$AMT" "$PEM")
MISC_NEURON_ID=$(stake Misc  "$AMT" "$PEM")

export AV_NEURON_ID OV_NEURON_ID OR_NEURON_ID

# use (345420 = 4 * 24 * 60 * 60 - 180) so that proposals are considered actionable 180 seconds after their submission (for testing purposes)
INIT_ARG="(record {
  alpha_vote_neuron_id              = ${AV_NEURON_ID};
  omega_vote_neuron_id              = ${OV_NEURON_ID};
  omega_reject_neuron_id            = ${OR_NEURON_ID};
  seconds_before_deadline_threshold = 345420
})"

echo "$INIT_ARG"                 # handy if you need to copy/paste it

echo Deploying alpha_backend canister locally, and initialising with local test neurons...

dfx deploy alpha_backend --argument "$INIT_ARG"

echo Configuring test neurons, and submitting test proposals. Please wait...

NEURON_IDS=("$AV_NEURON_ID" "$OV_NEURON_ID" "$OR_NEURON_ID" "$MOCK_D_QUORUM_NEURON_ID")
HOTKEY=$(dfx canister id alpha_backend)
FOLLOWER_NEURONS=("$AV_NEURON_ID")   # Omega neurons should not follow any neuron
FOLLOW_TOPICS=(0 4 14) # covers all topics (unspecified, governance, SNS & Neurons Fund)

{
  echo "identity private \"$PEM\";"
  echo

  # ── 3. AV neuron follows MISC_NEURON_ID on each topic in FOLLOW_TOPICS so that we can simulate the behaviour we get from following D-QUORUM (by triggering a vote on misc neuron)
  for follower in "${FOLLOWER_NEURONS[@]}"; do
    for topic in "${FOLLOW_TOPICS[@]}"; do
      cat <<EOF
call "rrkah-fqaaa-aaaaa-aaaaq-cai".manage_neuron(
  record {
    id = opt record { id = $follower : nat64 };
    command = opt variant {
      Follow = record {
        topic      = $topic : int32;
        followees  = vec { record { id = $MOCK_D_QUORUM_NEURON_ID }; };
      }
    }
  }
);
EOF
    done
  done

  # ---------- Add hot-key ----------------------------------------------------
  for id in "${NEURON_IDS[@]}"; do
    cat <<EOF
call "rrkah-fqaaa-aaaaa-aaaaq-cai".manage_neuron(
  record {
    id = opt record { id = $id : nat64 };
    command = opt variant {
      Configure = record {
        operation = opt variant {
          AddHotKey = record { new_hot_key = opt principal "$HOTKEY" }
        }
      }
    }
  }
);
EOF
  done

  echo

  # ---------- Increase dissolve delay (+1 year) -----------------------------
  for id in "${NEURON_IDS[@]}"; do
    cat <<EOF
call "rrkah-fqaaa-aaaaa-aaaaq-cai".manage_neuron(
  record {
    id = opt record { id = $id : nat64 };
    command = opt variant {
      Configure = record {
        operation = opt variant {
          IncreaseDissolveDelay = record {
            additional_dissolve_delay_seconds = 31536000   // +365 days
          }
        }
      }
    }
  }
);
EOF
  done

  cat <<EOF
call "rrkah-fqaaa-aaaaa-aaaaq-cai".manage_neuron(record{ id = opt record { id = $MOCK_D_QUORUM_NEURON_ID }; command = opt variant{ MakeProposal = record{ url = ""; title = opt "Known Neuron Proposal"; action = opt variant { RegisterKnownNeuron = record{ id = opt record { id = $MOCK_D_QUORUM_NEURON_ID }; known_neuron_data = opt record { name = "Test Known Neuron"; description = opt "Test proposal for a known neuron" } } }; summary = "Proposal summary text" } } });

call "rrkah-fqaaa-aaaaa-aaaaq-cai".manage_neuron(record{ id = opt record { id = $MISC_NEURON_ID }; command = opt variant{ MakeProposal = record{ url = ""; title = opt "Known Neuron Proposal"; action = opt variant { RegisterKnownNeuron = record{ id = opt record { id = $MISC_NEURON_ID }; known_neuron_data = opt record { name = "Test Known Neuron"; description = opt "Test proposal for a known neuron" } } }; summary = "Proposal summary text" } } });

call "rrkah-fqaaa-aaaaa-aaaaq-cai".manage_neuron(record{ id = opt record { id = $MOCK_D_QUORUM_NEURON_ID }; command = opt variant{ MakeProposal = record{ url = ""; title = opt ""; action = opt variant { ManageNeuron = record{ id = opt record { id = $MOCK_D_QUORUM_NEURON_ID }; command = opt variant{ Follow = record{ topic = 0; followees = vec { } } } } }; summary = "Test Neuron Management proposal, which should not be visible to the canister" } } });

EOF

} | ic-repl -r "$IC_URL" /dev/stdin

cat <<EOF

Neuron ids captured and exported:
  MISC_NEURON_ID=$MISC_NEURON_ID
  MOCK_D_QUORUM_NEURON_ID=$MOCK_D_QUORUM_NEURON_ID
  AV_NEURON_ID=$AV_NEURON_ID
  OV_NEURON_ID=$OV_NEURON_ID
  OR_NEURON_ID=$OR_NEURON_ID
EOF
