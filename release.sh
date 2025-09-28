#!/bin/sh

export PATH=/root/.local/share/dfx/bin:${PATH}

make build
dfx start --background
dfx deploy alpha_backend --argument='(record {
  alpha_vote_neuron_id              = 0;
  omega_vote_neuron_id              = 0;
  omega_reject_neuron_id            = 0;
  seconds_before_deadline_threshold = 0
})'
dfx canister info alpha_backend
dfx stop
