#!/usr/bin/env bash
# create_neurons_simple.sh
# Source it (  source ./create_neurons_simple.sh  ) to keep the variables.

set -euo pipefail

IC_URL=http://127.0.0.1:8080/
PEM=~/.config/dfx/identity/ident-1/identity.pem

{
  echo "identity private \"$PEM\";"
  echo

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
