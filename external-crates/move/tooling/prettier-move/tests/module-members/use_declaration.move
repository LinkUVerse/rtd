// options:
// printWidth: 50
// useModuleLabel: true
// autoGroupImports: module

module prettier::use_declaration;

use rtd::coin::Coin;
use rtd::coin::Coin as C;
use rtd::coin::{Self as c, Coin as C};
use rtd::coin::very_long_function_name_very_long_function_name as short_name;
use beep::staked_rtd::StakedRtd;

use rtd::transfer_policy::{Self as policy, TransferPolicy, TransferPolicyCap, TransferRequest};
use rtd::transfer_policy::TransferPolicyCap as cap;
use rtd::{
    transfer_policy::{TransferPolicy, TransferPolicyCap, TransferRequest, Kek as KEK},
    transfer_policy::TransferPolicyCap as cap,
};

public use fun my_custom_function_with_a_long_name as TransferPolicyCap.very_long_function_name;

friend has_been::here;

// will break before `as`
public use fun my_custom_function_with_a_long_name
    as TransferPolicyCap.very_long_function_name;
