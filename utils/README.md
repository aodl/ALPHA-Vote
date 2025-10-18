# Proposal Management Workflow

**This folder contains a collection of bash utilities** to simplify the process of submitting and reviewing threshold proposals. Changes cannot be made to the alpha_backend canister without team member consensus, which requires proposals to be submitted to the threshold canister. >50% of the team must accept the proposal for the change to take effect.

**A generally useful command** to run is to review a summary of the most recent proposals - `dfx canister call basbh-oyaaa-aaaar-qbxha-cai getProposalSummaries --network=IC` (or `--network=local` if you're reviewing test proposals that were submitted on a local replica). Note that `basbh-oyaaa-aaaar-qbxha-cai` is the threshold canister principal. Also note that this summary list won't display proposal payloads. You can drill into specific proposals with other commands. This process is massively simplified by utilising the following utility scripts.

**Refer to this readme for an explanation of how to**:
- [Submit any arbitrary proposal](/utils/README.md#submit-any-arbitrary-proposal)
- [Retrieve a submitted proposal, inspect the details, and vote to accept/reject](/utils/README.md#retrieve-a-submitted-proposal-inspect-the-details-and-vote-to-acceptreject)
- [Submit a proposal to upgrade the canister](/utils/README.md#submit-a-proposal-to-upgrade-the-canister)
- [Retrieve a canister upgrade proposal and compute the hash](/utils/README.md#retrieve-a-canister-upgrade-proposal-and-compute-the-hash)

### Submit any arbitrary proposal

You can submit any arbitrary proposal, by specifying a canister (this would be with the alpha_backend canister or the threshold canister itself), a method to call on that canister, and optionally an argument to pass to that method.

This is simplified by using the `submit_proposal_canister_call.sh` script. Be sure to first execute `chmod +x submit_proposal_canister_call.sh` to allow execution of the bash script.

Call `./submit_proposal_canister_call.sh` to retrieve usage information (the same approach can be used for other scripts). ->

```console
Usage: ./submit_proposal_canister_call.sh arg1_network arg2_proposalSummary arg3_targetCanister arg4_targetMethod arg5_methodArg
```

The submitted proposal id will be provided upon successful submission. e.g.

```console
root@BuildMachine:/home/ALPHA-Vote/utils# ./submit_proposal_canister_call.sh ic "Remove Lorimer as controller of the threshold canister" aaaaa-aa update_settings '(record {
  canister_id = principal "basbh-oyaaa-aaaar-qbxha-cai";
  settings = record {
    controllers = opt vec { principal "basbh-oyaaa-aaaar-qbxha-cai" };
    compute_allocation = null;
    memory_allocation = null;
    freezing_threshold = null
  }
})'

Submit proposal to threshold canister 'basbh-oyaaa-aaaar-qbxha-cai'
to call 'update_settings' on canister 'aaaaa-aa' on the ic network?
(y/n):
y
Proceeding...
About to encode METHOD_ARG (record {
  canister_id = principal "basbh-oyaaa-aaaar-qbxha-cai";
  settings = record {
    controllers = opt vec { principal "basbh-oyaaa-aaaar-qbxha-cai" };
    compute_allocation = null;
    memory_allocation = null;
    freezing_threshold = null
  }
})
Encoded method args: vec {68; 73; 68; 76; 4; 108; 2; 179; 196; 177; 242; 4; 104; 227; 249; 245; 217; 8; 1; 108; 4; 192; 207; 242; 113; 127; 215; 224; 155; 144; 2; 2; 222; 235; 181; 169; 14; 127; 168; 130; 172; 198; 15; 127; 110; 3; 109; 104; 1; 0; 1; 10; 0; 0; 0; 0; 2; 48; 13; 206; 1; 1; 1; 1; 1; 10; 0; 0; 0; 0; 2; 48; 13; 206; 1; 1}

Please enter the passphrase for your identity: [hidden]
Decryption complete.
(1 : nat)

=== Done! ===
```

Here's an example using a different type of proposal (instead of removing a controller, this one sets the principals with voting rights).

```console
root@BuildMachine:/home/ALPHA-Vote/utils# ./submit_proposal_canister_call.sh ic "Update threshold principals to include all 15 members" basbh-oyaaa-aaaar-qbxha-cai setSigners '(vec {principal "zkkkd-i34qc-367ln-e2u7o-ezznu-dkfqh-gtfvz-cviph-6qa4v-evtfs-wqe"; principal "zeu5w-lsfxj-o4f5b-xl4sg-enqva-ez4wa-2nlbb-ufigi-fwnmv-zrfok-uae"; principal "j2tnr-f5tmm-afnyl-762n7-o272x-ji2xi-bcpld-ihimy-fw52d-2zqov-xae"; principal "koiza-s6kz2-m45zq-4lrn7-4v65m-6zemu-neoxj-vz6cb-ouolw-rrawv-mae"; principal "5l35f-cafxz-s42hj-lrq7g-wpikv-rptdg-k2vk5-aaa6k-qfvib-h5umc-nae"; principal "bn6wo-xpofx-5va6n-knhsi-d26er-6oxej-a5m3i-i5yh7-h3il7-s65zr-lae"; principal "j2tnr-f5tmm-afnyl-762n7-o272x-ji2xi-bcpld-ihimy-fw52d-2zqov-xae"; principal "pib55-fsiwt-ftxf3-a6e7q-ed7dm-qfbgq-tdld3-jrotf-7y5bf-xsyju-uqe"; principal "4e6g2-eoooo-h2lec-3h725-hvmmc-fvgsd-qakd3-qsj44-6dlaw-p5ngz-mae"; principal "mrzkb-iqvzd-crtjw-g2fai-hapm4-hlchs-exq4o-in7jm-szk73-o7bjn-cqe"; principal "hfbtd-e2vzk-rvwfx-c55l3-tsuue-oizir-hl4bg-tajby-skikk-iefse-fqe"; principal "vwng4-j5dgs-e5kv2-ofyq2-hc4be-7u2fn-mmncn-u7dhj-nzkyq-vktfa-xqe"; principal "yvi2m-qclpo-iof7c-xbzh5-4g2hb-i36yy-yx7i2-iczo2-oei56-ldao3-rae"; principal "2suit-vsnnb-qhqlz-jjkhw-6boyv-qg55o-aastu-vt7qc-47vnc-xkwci-qqe"; principal "antzc-h747k-lqald-mrvxf-cqtxq-wa3az-7pkrk-zowgt-jarbq-yotxs-eae"})'

Submit proposal to threshold canister 'basbh-oyaaa-aaaar-qbxha-cai'
to call 'setSigners' on canister 'basbh-oyaaa-aaaar-qbxha-cai' on the ic network?
(y/n):
y
Proceeding...
About to encode METHOD_ARG (vec {principal "zkkkd-i34qc-367ln-e2u7o-ezznu-dkfqh-gtfvz-cviph-6qa4v-evtfs-wqe"; principal "zeu5w-lsfxj-o4f5b-xl4sg-enqva-ez4wa-2nlbb-ufigi-fwnmv-zrfok-uae"; principal "j2tnr-f5tmm-afnyl-762n7-o272x-ji2xi-bcpld-ihimy-fw52d-2zqov-xae"; principal "koiza-s6kz2-m45zq-4lrn7-4v65m-6zemu-neoxj-vz6cb-ouolw-rrawv-mae"; principal "5l35f-cafxz-s42hj-lrq7g-wpikv-rptdg-k2vk5-aaa6k-qfvib-h5umc-nae"; principal "bn6wo-xpofx-5va6n-knhsi-d26er-6oxej-a5m3i-i5yh7-h3il7-s65zr-lae"; principal "j2tnr-f5tmm-afnyl-762n7-o272x-ji2xi-bcpld-ihimy-fw52d-2zqov-xae"; principal "pib55-fsiwt-ftxf3-a6e7q-ed7dm-qfbgq-tdld3-jrotf-7y5bf-xsyju-uqe"; principal "4e6g2-eoooo-h2lec-3h725-hvmmc-fvgsd-qakd3-qsj44-6dlaw-p5ngz-mae"; principal "mrzkb-iqvzd-crtjw-g2fai-hapm4-hlchs-exq4o-in7jm-szk73-o7bjn-cqe"; principal "hfbtd-e2vzk-rvwfx-c55l3-tsuue-oizir-hl4bg-tajby-skikk-iefse-fqe"; principal "vwng4-j5dgs-e5kv2-ofyq2-hc4be-7u2fn-mmncn-u7dhj-nzkyq-vktfa-xqe"; principal "yvi2m-qclpo-iof7c-xbzh5-4g2hb-i36yy-yx7i2-iczo2-oei56-ldao3-rae"; principal "2suit-vsnnb-qhqlz-jjkhw-6boyv-qg55o-aastu-vt7qc-47vnc-xkwci-qqe"; principal "antzc-h747k-lqald-mrvxf-cqtxq-wa3az-7pkrk-zowgt-jarbq-yotxs-eae"})
Encoded method args: vec {68; 73; 68; 76; 1; 109; 104; 1; 0; 15; 1; 29; 124; 128; 183; 239; 173; 164; 213; 62; 226; 103; 45; 160; 212; 88; 28; 211; 45; 114; 42; 161; 231; 244; 1; 202; 146; 179; 44; 173; 2; 1; 29; 69; 186; 93; 194; 244; 55; 95; 36; 98; 54; 21; 1; 51; 203; 3; 77; 88; 67; 66; 160; 200; 45; 154; 202; 230; 37; 114; 168; 2; 1; 29; 179; 99; 0; 86; 225; 127; 246; 155; 247; 107; 250; 186; 81; 171; 160; 34; 122; 198; 131; 161; 152; 45; 187; 161; 235; 48; 117; 110; 2; 1; 29; 202; 206; 153; 206; 230; 28; 92; 91; 254; 87; 221; 103; 178; 70; 81; 164; 117; 211; 92; 248; 65; 117; 28; 187; 70; 32; 181; 88; 2; 1; 29; 5; 190; 101; 205; 29; 43; 140; 62; 107; 61; 10; 172; 95; 49; 153; 90; 170; 186; 0; 3; 202; 129; 106; 128; 159; 180; 96; 154; 2; 1; 29; 238; 45; 251; 80; 121; 170; 105; 228; 129; 235; 196; 143; 157; 114; 36; 29; 102; 208; 142; 224; 255; 62; 208; 191; 203; 221; 204; 86; 2; 1; 29; 179; 99; 0; 86; 225; 127; 246; 155; 247; 107; 250; 186; 81; 171; 160; 34; 122; 198; 131; 161; 152; 45; 187; 161; 235; 48; 117; 110; 2; 1; 29; 72; 180; 203; 59; 151; 96; 241; 63; 2; 15; 227; 100; 10; 19; 66; 99; 88; 246; 152; 186; 101; 254; 58; 18; 222; 88; 77; 41; 2; 1; 29; 206; 115; 143; 165; 144; 91; 63; 245; 211; 213; 140; 17; 106; 105; 14; 0; 80; 247; 9; 39; 156; 240; 214; 11; 63; 173; 54; 88; 2; 1; 29; 21; 200; 197; 25; 166; 198; 209; 64; 131; 129; 236; 225; 214; 35; 200; 151; 135; 28; 134; 253; 44; 150; 85; 253; 187; 225; 75; 69; 2; 1; 29; 85; 202; 163; 91; 22; 226; 239; 87; 185; 202; 148; 35; 145; 148; 68; 235; 224; 77; 48; 36; 56; 146; 144; 165; 32; 133; 145; 11; 2; 1; 29; 163; 52; 137; 213; 87; 78; 46; 33; 163; 139; 129; 39; 233; 162; 181; 140; 104; 155; 79; 140; 233; 110; 85; 136; 85; 83; 40; 47; 2; 1; 29; 75; 123; 144; 226; 252; 87; 14; 79; 222; 27; 71; 10; 55; 236; 99; 23; 250; 52; 129; 101; 218; 113; 17; 223; 44; 96; 118; 226; 2; 1; 29; 77; 104; 96; 120; 47; 41; 74; 143; 111; 5; 216; 172; 13; 222; 184; 0; 148; 233; 89; 254; 2; 231; 234; 209; 93; 86; 18; 33; 2; 1; 29; 252; 250; 151; 0; 44; 108; 141; 110; 81; 66; 119; 133; 129; 176; 103; 239; 84; 85; 151; 88; 211; 72; 34; 24; 97; 211; 188; 136; 2}

Please enter the passphrase for your identity: [hidden]
Decryption complete.
(2 : nat)

=== Done! ===
```

### Retrieve a submitted proposal, inspect the details, and vote to accept/reject

If you've followed the steps above, the next thing you'll want to do is confirm that your proposal looks okay. Your team members will also need to carry out this step in order to vote on that proposal.

Use the `retrieve_proposal_canister_call.sh` script as follows. Be sure to first execute `chmod +x retrieve_proposal_canister_call.sh` to allow execution of the bash script.

- Execute `./retrieve_proposal_canister_call.sh local <proposal_id>` if your testing a local fake proposal
- Execute `./retrieve_proposal_canister_call.sh ic <proposal_id>` if your reviewing a live production proposal

e.g. Here's what this would look like for proposal 1 submitted in the examples above.

```console
root@BuildMachine:/home/ALPHA-Vote/utils# ./retrieve_proposal_canister_call.sh ic 1
Attempting to retrieve proposal id 1 ...
Please enter the passphrase for your identity: [hidden]
Decryption complete.

( opt record { id = 1 : nat; memo = "Remove Lorimer as controller of the threshold canister"; state = record { no = 0 : nat; yes = 1 : nat; result = null; active = true; votes = vec { record { 1_760_713_546 : nat; principal "zkkkd-i34qc-367ln-e2u7o-ezznu-dkfqh-gtfvz-cviph-6qa4v-evtfs-wqe"; }; }; }; payload = record { principal "aaaaa-aa"; "update_settings"; blob "\44\49\44\4c\04\6c\02\b3\c4\b1\f2\04\68\e3\f9\f5\d9\08\01\6c\04\c0\cf\f2\71\7f\d7\e0\9b\90\02\02\de\eb\b5\a9\0e\7f\a8\82\ac\c6\0f\7f\6e\03\6d\68\01\00\01\0a\00\00\00\00\02\30\0d\ce\01\01\01\01\01\0a\00\00\00\00\02\30\0d\ce\01\01"; }; }, )

Attempting to decode payload blob ...
Please enter the passphrase for your identity: [hidden]
Decryption complete.
(
  record {
    1_313_628_723 = principal "basbh-oyaaa-aaaar-qbxha-cai";
    2_336_062_691 = record {
      238_856_128 = null : null;
      570_880_087 = opt vec { principal "basbh-oyaaa-aaaar-qbxha-cai" };
      3_844_961_758 = null : null;
      4_174_053_672 = null : null;
    };
  },
)

Proposal appears to be active. Type 'accept' or 'reject' and then press ENTER to vote, or type anything else to exit...
accept
Accepting the proposal...
Please enter the passphrase for your identity: [hidden]
Decryption complete.
()
```

It's worth noting that the script doesn't know the candid type for updating canister settings, so the field names are just displayed as numbers. If this is problematic and you'd like to see the field name, to get around this you can grab the payload blob string from the output above, then strip out the '\\' delimiters, and feed it to `didc` yourself, specifying the location of the `.did` file for type that's used as an argument.

e.g.

```console
root@BuildMachine:/home/ALPHA-Vote/utils# didc decode --defs <(curl -sSL https://raw.githubusercontent.com/dfinity/cdk-rs/HEAD/ic-management-canister-types/tests/ic.did) --types '(record { canister_id : principal; settings : canister_settings })' '4449444c046c02b3c4b1f20468e3f9f5d908016c04c0cff2717fd7e09b900202deebb5a90e7fa882acc60f7f6e036d680100010a0000000002300dce01010101010a0000000002300dce0101'
(
  record {
    canister_id = principal "basbh-oyaaa-aaaar-qbxha-cai";
    settings = record {
      freezing_threshold = null;
      wasm_memory_threshold = null;
      environment_variables = null;
      controllers = opt vec { principal "basbh-oyaaa-aaaar-qbxha-cai" };
      reserved_cycles_limit = null;
      log_visibility = null;
      wasm_memory_limit = null;
      memory_allocation = null;
      compute_allocation = null;
    };
  },
)
```

Note that not all proposal use complex types that require .did files. Take proposal 2 in the example submissions above. Here's what it would looke like to review and vote on that proposal.

```console
root@BuildMachine:/home/CO_DELTA/CODELTA/alpha/alpha/utils# ./retrieve_proposal_canister_call.sh ic 2
Attempting to retrieve proposal id 2 ...
Please enter the passphrase for your identity: [hidden]
Decryption complete.

( opt record { id = 2 : nat; memo = "Update threshold principals to include all 15 members"; state = record { no = 0 : nat; yes = 1 : nat; result = null; active = true; votes = vec { record { 1_760_715_554 : nat; principal "zkkkd-i34qc-367ln-e2u7o-ezznu-dkfqh-gtfvz-cviph-6qa4v-evtfs-wqe"; }; }; }; payload = record { principal "basbh-oyaaa-aaaar-qbxha-cai"; "setSigners"; blob "\44\49\44\4c\01\6d\68\01\00\0f\01\1d\7c\80\b7\ef\ad\a4\d5\3e\e2\67\2d\a0\d4\58\1c\d3\2d\72\2a\a1\e7\f4\01\ca\92\b3\2c\ad\02\01\1d\45\ba\5d\c2\f4\37\5f\24\62\36\15\01\33\cb\03\4d\58\43\42\a0\c8\2d\9a\ca\e6\25\72\a8\02\01\1d\b3\63\00\56\e1\7f\f6\9b\f7\6b\fa\ba\51\ab\a0\22\7a\c6\83\a1\98\2d\bb\a1\eb\30\75\6e\02\01\1d\ca\ce\99\ce\e6\1c\5c\5b\fe\57\dd\67\b2\46\51\a4\75\d3\5c\f8\41\75\1c\bb\46\20\b5\58\02\01\1d\05\be\65\cd\1d\2b\8c\3e\6b\3d\0a\ac\5f\31\99\5a\aa\ba\00\03\ca\81\6a\80\9f\b4\60\9a\02\01\1d\ee\2d\fb\50\79\aa\69\e4\81\eb\c4\8f\9d\72\24\1d\66\d0\8e\e0\ff\3e\d0\bf\cb\dd\cc\56\02\01\1d\b3\63\00\56\e1\7f\f6\9b\f7\6b\fa\ba\51\ab\a0\22\7a\c6\83\a1\98\2d\bb\a1\eb\30\75\6e\02\01\1d\48\b4\cb\3b\97\60\f1\3f\02\0f\e3\64\0a\13\42\63\58\f6\98\ba\65\fe\3a\12\de\58\4d\29\02\01\1d\ce\73\8f\a5\90\5b\3f\f5\d3\d5\8c\11\6a\69\0e\00\50\f7\09\27\9c\f0\d6\0b\3f\ad\36\58\02\01\1d\15\c8\c5\19\a6\c6\d1\40\83\81\ec\e1\d6\23\c8\97\87\1c\86\fd\2c\96\55\fd\bb\e1\4b\45\02\01\1d\55\ca\a3\5b\16\e2\ef\57\b9\ca\94\23\91\94\44\eb\e0\4d\30\24\38\92\90\a5\20\85\91\0b\02\01\1d\a3\34\89\d5\57\4e\2e\21\a3\8b\81\27\e9\a2\b5\8c\68\9b\4f\8c\e9\6e\55\88\55\53\28\2f\02\01\1d\4b\7b\90\e2\fc\57\0e\4f\de\1b\47\0a\37\ec\63\17\fa\34\81\65\da\71\11\df\2c\60\76\e2\02\01\1d\4d\68\60\78\2f\29\4a\8f\6f\05\d8\ac\0d\de\b8\00\94\e9\59\fe\02\e7\ea\d1\5d\56\12\21\02\01\1d\fc\fa\97\00\2c\6c\8d\6e\51\42\77\85\81\b0\67\ef\54\55\97\58\d3\48\22\18\61\d3\bc\88\02"; }; }, )

Attempting to decode payload blob ...
Please enter the passphrase for your identity: [hidden]
Decryption complete.
(
  vec { principal "zkkkd-i34qc-367ln-e2u7o-ezznu-dkfqh-gtfvz-cviph-6qa4v-evtfs-wqe"; principal "zeu5w-lsfxj-o4f5b-xl4sg-enqva-ez4wa-2nlbb-ufigi-fwnmv-zrfok-uae"; principal "j2tnr-f5tmm-afnyl-762n7-o272x-ji2xi-bcpld-ihimy-fw52d-2zqov-xae"; principal "koiza-s6kz2-m45zq-4lrn7-4v65m-6zemu-neoxj-vz6cb-ouolw-rrawv-mae"; principal "5l35f-cafxz-s42hj-lrq7g-wpikv-rptdg-k2vk5-aaa6k-qfvib-h5umc-nae"; principal "bn6wo-xpofx-5va6n-knhsi-d26er-6oxej-a5m3i-i5yh7-h3il7-s65zr-lae"; principal "j2tnr-f5tmm-afnyl-762n7-o272x-ji2xi-bcpld-ihimy-fw52d-2zqov-xae"; principal "pib55-fsiwt-ftxf3-a6e7q-ed7dm-qfbgq-tdld3-jrotf-7y5bf-xsyju-uqe"; principal "4e6g2-eoooo-h2lec-3h725-hvmmc-fvgsd-qakd3-qsj44-6dlaw-p5ngz-mae"; principal "mrzkb-iqvzd-crtjw-g2fai-hapm4-hlchs-exq4o-in7jm-szk73-o7bjn-cqe"; principal "hfbtd-e2vzk-rvwfx-c55l3-tsuue-oizir-hl4bg-tajby-skikk-iefse-fqe"; principal "vwng4-j5dgs-e5kv2-ofyq2-hc4be-7u2fn-mmncn-u7dhj-nzkyq-vktfa-xqe"; principal "yvi2m-qclpo-iof7c-xbzh5-4g2hb-i36yy-yx7i2-iczo2-oei56-ldao3-rae"; principal "2suit-vsnnb-qhqlz-jjkhw-6boyv-qg55o-aastu-vt7qc-47vnc-xkwci-qqe"; principal "antzc-h747k-lqald-mrvxf-cqtxq-wa3az-7pkrk-zowgt-jarbq-yotxs-eae";},
)

Proposal appears to be active. Type 'accept' or 'reject' and then press ENTER to vote, or type anything else to exit...
accept
Accepting the proposal...
Please enter the passphrase for your identity: [hidden]
Decryption complete.
()
```

### Submit a proposal to upgrade the canister

If there's ever a need to update the alpha_backend canister (or rarely the threshold canister), it will need to be done via a proposal, as no individual has control over the canisters.

After you've built and tested the changes, you need to provide the file path for the WASM, along with an optional upgrade argument.

This is simplified by using the `submit_proposal_canister_upgrade.sh` script. Be sure to first execute `chmod +x submit_proposal_canister_upgrade.sh` to allow execution of the bash script.

Call `./submit_proposal_canister_upgrade.sh` to retrieve usage information (the same approach can be used for other scripts). ->

```console
Usage: ./submit_proposal_canister_upgrade.sh arg1_network arg2_upgradeCanisterPrincipal arg3_wasmFilePath arg4_upgradeArg
```

Note that various files are generated in the `Utils/canister_upgrade_pipeline/` folder just in case you need to inspect the stages of encoding arguments, and/or converting the WASM bytes to a format that's suitable for passing into the proposal (a vec of backslash-escaped hex bytes).

**Important:** If there's ever a need to upgrade the threshold canister itself, the same approach can be used, however there are some precautions that should be taken. A botched upgrade to the threshold cansiter can result in a bricked system. Before upgrading the threshold canister (if this is ever actually needed) a second threshold canister should first be set up, with identical configuration and control priviledges. If the upgrade of one of these threshold canisters fails in a way that leaves the canister in a broken state, the healthy threshold canister can be used to rectify the situation. Setting up the second threshold canister will require several proposals to be submitted to the original threshold canister first (to give the second threshold canister the same level of control i.e. updating canister settings to add it as a controller. Refer to the main README in this repo for the exact settings necessary).

### Retrieve a canister upgrade proposal and compute the hash 

This script is particularly cool, and it massively simplifies the process of reviewing and verifying canister upgrade proposals. e.g.

```console
root@BuildMachine:/home/ALPHA-Vote/utils# ./retrieve_proposal_canister_upgrade.sh local 4

Attempting to retrieve canister arg for upgrade proposal id 4 ...
No canister args to decode

Attempting to retrieve canister WASM for upgrade proposal id 4 ...
Please enter the passphrase for your identity: [hidden]
Decryption complete.
Saved WASM to canister_upgrade_pipeline/module.wasm

Attempting to compute WASM Sha256...
ef860db565eed8e2322c60a9931300d05d20826adf1e00a96cb8a294cfe4ea85  canister_upgrade_pipeline/module.wasm

=== Done! ===

If the proposal is active you can accept or reject by executing 'dfx canister call basbh-oyaaa-aaaar-qbxha-cai accept 4 --network=local' or 'dfx canister call basbh-oyaaa-aaaar-qbxha-cai reject 4 --network=local'. You can confirm your vote was received by calling 'dfx canister call basbh-oyaaa-aaaar-qbxha-cai getProposal 4 --network=local' and observing the vote tally
```
