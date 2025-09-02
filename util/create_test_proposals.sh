#!/usr/bin/env bash
# create_neurons_simple.sh
# Source it (  source ./create_neurons_simple.sh  ) to keep the variables.

set -euo pipefail

IC_URL=http://127.0.0.1:8080/
PEM=~/.config/dfx/identity/ident-1/identity.pem
AMT=3

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

MISC_NEURON_ID=$(stake misc "$AMT" "$PEM")

{
  echo "identity private \"$PEM\";"
  echo

  cat <<EOF
call "rrkah-fqaaa-aaaaa-aaaaq-cai".manage_neuron(
  record {
    id = opt record { id = $MISC_NEURON_ID : nat64 };
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

call "rrkah-fqaaa-aaaaa-aaaaq-cai".manage_neuron(record{ id = opt record { id = $MISC_NEURON_ID }; command = opt variant{ MakeProposal = record{ url = ""; title = opt "Known Neuron Proposal"; action = opt variant { RegisterKnownNeuron = record{ id = opt record { id = $MISC_NEURON_ID }; known_neuron_data = opt record { name = "Test Known Neuron"; description = opt "Test proposal for a known neuron" } } }; summary = "Proposal summary text" } } });

call "rrkah-fqaaa-aaaaa-aaaaq-cai".manage_neuron(record{ id = opt record { id = $MISC_NEURON_ID }; command = opt variant{ MakeProposal = record{ url = ""; title = opt ""; action = opt variant { ManageNeuron = record{ id = opt record { id = $MISC_NEURON_ID }; command = opt variant{ Follow = record{ topic = 0; followees = vec { } } } } }; summary = "Test Neuron Management proposal, which should not be visible to the canister" } } });

EOF

} | ic-repl -r "$IC_URL" /dev/stdin

cat <<EOF

Neuron ids captured and exported:
  MISC_NEURON_ID=$MISC_NEURON_ID
EOF
