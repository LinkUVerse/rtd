// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[test_only]
module rtd_system::rewards_distribution_tests;

use std::unit_test::assert_eq;
use rtd::address;
use rtd_system::rtd_system::RtdSystemState;
use rtd_system::test_runner;
use rtd_system::validator_builder;

const VALIDATOR_ADDR_1: address = @01;
const VALIDATOR_ADDR_2: address = @02;
const VALIDATOR_ADDR_3: address = @03;
const VALIDATOR_ADDR_4: address = @04;

const STAKER_ADDR_1: address = @42;
const STAKER_ADDR_2: address = @43;
const STAKER_ADDR_3: address = @44;
const STAKER_ADDR_4: address = @45;

const MIST_PER_RTD: u64 = 1_000_000_000;

#[test]
fun validator_rewards() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1000)
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    let opts = runner.advance_epoch_opts().computation_charge(100);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // check rewards distribution, 1:2:3:4
    runner.system_tx!(|system, _| {
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_1), 125 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_2), 225 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_3), 325 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_4), 425 * MIST_PER_RTD);
    });

    runner.set_sender(VALIDATOR_ADDR_2).stake_with(VALIDATOR_ADDR_2, 720);

    let opts = runner.advance_epoch_opts().computation_charge(100);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // check rewards distribution, given that validator 2 has 920 RTD of stake now
    runner.system_tx!(|system, _| {
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_1), 150 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_2), 970 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_3), 350 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_4), 450 * MIST_PER_RTD);
    });

    runner.finish();
}

#[test]
fun stake_subsidy() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1_000_000_000)
        .validators(vector[
            validator_builder::new().initial_stake(100_000_000).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200_000_000).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300_000_000).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400_000_000).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    let opts = runner.advance_epoch_opts().computation_charge(100);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    runner.system_tx!(|system, _| {
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_1), 100_000_025 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_2), 200_000_025 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_3), 300_000_025 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_4), 400_000_025 * MIST_PER_RTD);
    });

    runner.finish();
}

#[test]
fun stake_rewards() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1000)
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    let validators = runner.genesis_validator_addresses();

    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 200);
    runner.set_sender(STAKER_ADDR_2).stake_with(VALIDATOR_ADDR_2, 100);

    // advance epoch so that the stake is active
    runner.advance_epoch(option::none()).destroy_for_testing();

    // check the total stake amount
    runner.system_tx!(|system, _| {
        // total stake
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_1), 300 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_2), 300 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_3), 300 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_4), 400 * MIST_PER_RTD);
    });

    // check total stake and rewards for each validator
    assert_stake_rewards_for_addresses(
        &mut runner,
        validators,
        vector[100 * MIST_PER_RTD, 200 * MIST_PER_RTD, 300 * MIST_PER_RTD, 400 * MIST_PER_RTD],
    );

    let opts = runner.advance_epoch_opts().computation_charge(120);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // check total stake and rewards for each validator
    assert_stake_rewards_for_addresses(
        &mut runner,
        validators,
        vector[110 * MIST_PER_RTD, 220 * MIST_PER_RTD, 330 * MIST_PER_RTD, 430 * MIST_PER_RTD],
    );

    runner.set_sender(STAKER_ADDR_1).unstake(0);
    runner.set_sender(STAKER_ADDR_2).stake_with(VALIDATOR_ADDR_1, 600);

    // Each pool gets 30 RTD.
    let opts = runner.advance_epoch_opts().computation_charge(120);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    assert_stake_rewards_for_addresses(
        &mut runner,
        validators,
        vector[140 * MIST_PER_RTD, 240 * MIST_PER_RTD, 360 * MIST_PER_RTD, 460 * MIST_PER_RTD],
    );

    // staker 1 receives only 20 RTD of rewards, not 40 since we are using pre-epoch exchange rate.
    assert_eq!(runner.set_sender(STAKER_ADDR_1).rtd_balance(), 220 * MIST_PER_RTD);

    // staker 2 receives 20 RTD of rewards.
    runner.set_sender(STAKER_ADDR_2).unstake(0);
    assert_eq!(runner.set_sender(STAKER_ADDR_2).rtd_balance(), 120 * MIST_PER_RTD);

    let opts = runner.advance_epoch_opts().computation_charge(40);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // unstake 600 principal RTD
    runner.set_sender(STAKER_ADDR_2).unstake(0);

    // additional 600 RTD of principal and 46 RTD of rewards withdrawn to Coin<RTD>
    // For this stake, the staking exchange rate is 100 : 140 and the unstaking
    // exchange rate is 528 : 750 -ish so the total rtd withdraw will be:
    // (600 * 100 / 140) * 750 / 528 = ~608. Together with the 120 RTD we already have,
    // that would be about 728 RTD.
    // TODO: Come up with better numbers and clean it up!
    assert_eq!(runner.set_sender(STAKER_ADDR_2).rtd_balance(), 728108108107);

    runner.finish();
}

#[test]
fun stake_tiny_rewards() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1_000_000_000)
        .validators(vector[
            validator_builder::new().initial_stake(100_000_000).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200_000_000).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300_000_000).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400_000_000).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 200000000);

    let opts = runner.advance_epoch_opts().computation_charge(150000);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // stake a small amount
    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 10);
    let opts = runner.advance_epoch_opts().computation_charge(130);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // unstake the stakes
    runner.set_sender(STAKER_ADDR_1).unstake(1);

    // and advance epoch should succeed
    let opts = runner.advance_epoch_opts().computation_charge(150);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    runner.finish();
}

#[test]
fun validator_commission() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1000)
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    let validators = runner.genesis_validator_addresses();

    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 100);
    runner.set_sender(STAKER_ADDR_2).stake_with(VALIDATOR_ADDR_2, 100);

    // validator 2 now has 20% commission
    // advance epoch to apply stake and update commission rate
    runner
        .set_sender(VALIDATOR_ADDR_2)
        .system_tx!(|system, ctx| system.request_set_commission_rate(20_00, ctx))
        .advance_epoch(option::none())
        .destroy_for_testing();

    // V1: 200, V2: 300, V3: 300, V4: 400
    runner.set_sender(VALIDATOR_ADDR_2).system_tx!(|system, _| {
        // check stakes
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_1), 200 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_2), 300 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_3), 300 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_4), 400 * MIST_PER_RTD);
    });

    let opts = runner.advance_epoch_opts().computation_charge(120);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // V1: 230, V2: 330, V3: 330, V4: 430
    runner.system_tx!(|system, _| {
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_1), 230 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_2), 330 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_3), 330 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_4), 430 * MIST_PER_RTD);
    });

    assert_stake_rewards_for_addresses(
        &mut runner,
        vector[STAKER_ADDR_1, STAKER_ADDR_2],
        vector[115 * MIST_PER_RTD, 108 * MIST_PER_RTD],
    );

    assert_stake_rewards_for_addresses(
        &mut runner,
        validators,
        vector[115 * MIST_PER_RTD, 222 * MIST_PER_RTD, 330 * MIST_PER_RTD, 430 * MIST_PER_RTD],
    );

    // validator 1 now has 10% commission
    runner
        .set_sender(VALIDATOR_ADDR_1)
        .system_tx!(|system, ctx| system.request_set_commission_rate(10_00, ctx))
        .advance_epoch(option::none())
        .destroy_for_testing();

    let opts = runner.advance_epoch_opts().computation_charge(240);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();
    runner.system_tx!(|system, _| {
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_1), 290 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_2), 390 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_3), 390 * MIST_PER_RTD);
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_4), 490 * MIST_PER_RTD);
    });

    // Staker 1 rewards in the recent distribution is 0.9 x 30 = 27 RTD
    // Validator 1 rewards in the recent distribution is 60 - 27 = 33 RTD

    // Staker 2 amounts for 0.8 * 60 * (108 / 330) + 108 = 123.709 RTD
    // Validator 2 amounts for 390 - 123.709 = 266.291 RTD

    assert_stake_rewards_for_addresses(
        &mut runner,
        vector[STAKER_ADDR_1, STAKER_ADDR_2],
        vector[142 * MIST_PER_RTD, 123709090909],
    );

    assert_stake_rewards_for_addresses(
        &mut runner,
        validators,
        vector[148 * MIST_PER_RTD, 266290909091, 390 * MIST_PER_RTD, 490 * MIST_PER_RTD],
    );

    runner.finish();
}

#[test]
fun rewards_slashing() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1000)
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    let validators = runner.genesis_validator_addresses();

    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 100);
    runner.set_sender(STAKER_ADDR_2).stake_with(VALIDATOR_ADDR_2, 100);
    runner.advance_epoch(option::none()).destroy_for_testing();

    vector[VALIDATOR_ADDR_1, VALIDATOR_ADDR_3, VALIDATOR_ADDR_4].do!(|validator_address| {
        runner.set_sender(validator_address).report_validator(VALIDATOR_ADDR_2);
    });

    let opts = runner.advance_epoch_opts().computation_charge(3600).reward_slashing_rate(1000);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // Without reward slashing, the validator's stakes should be [100+450, 200+600, 300+900, 400+900]
    // after the last epoch advancement.
    // Since 60 RTD, or 10% of validator_2's rewards (600) are slashed, she only has 800 - 60 = 740 now.
    // There are in total 90 RTD of rewards slashed (60 from the validator, and 30 from her staker)
    // so the unslashed validators each get their share of additional rewards, which is 30.
    assert_stake_rewards_for_addresses(
        &mut runner,
        validators,
        vector[565 * MIST_PER_RTD, 740 * MIST_PER_RTD, 1230 * MIST_PER_RTD, 1330 * MIST_PER_RTD],
    );

    runner.set_sender(STAKER_ADDR_1).unstake(0);
    runner.set_sender(STAKER_ADDR_2).unstake(0);

    assert_eq!(runner.set_sender(STAKER_ADDR_1).rtd_balance(), 565 * MIST_PER_RTD);
    assert_eq!(runner.set_sender(STAKER_ADDR_2).rtd_balance(), 370 * MIST_PER_RTD);

    runner.finish();
}

#[test]
fun entire_rewards_slashing() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1000)
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    let validators = runner.genesis_validator_addresses();

    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 100);
    runner.set_sender(STAKER_ADDR_2).stake_with(VALIDATOR_ADDR_2, 100);
    runner.advance_epoch(option::none()).destroy_for_testing();

    // validator_2 is reported by 3 other validators, so 75% of total stake.
    vector[VALIDATOR_ADDR_1, VALIDATOR_ADDR_3, VALIDATOR_ADDR_4].do!(|validator_address| {
        runner.set_sender(validator_address).report_validator(VALIDATOR_ADDR_2);
    });

    // 3600 RTD of total rewards, 100% reward slashing.
    // So validator_2 is the only one whose rewards should get slashed.
    let opts = runner.advance_epoch_opts().computation_charge(3600).reward_slashing_rate(10_000);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // Without reward slashing, the validator's stakes should be [100+450, 200+600, 300+900, 400+900]
    // after the last epoch advancement.
    // The entire rewards of validator 2's staking pool are slashed, which is 900 RTD.
    // so the unslashed validators each get their share of additional rewards, which is 300.
    assert_stake_rewards_for_addresses(
        &mut runner,
        validators,
        vector[
            (550 + 150) * MIST_PER_RTD,
            200 * MIST_PER_RTD,
            (1200 + 300) * MIST_PER_RTD,
            (1300 + 300) * MIST_PER_RTD,
        ],
    );

    // Unstake so we can check the stake rewards as well.
    runner.set_sender(STAKER_ADDR_1).unstake(0);
    runner.set_sender(STAKER_ADDR_2).unstake(0);

    // Same analysis as above. Staker 1 has 150 additional RTD, and since all of staker 2's rewards are slashed she only gets back her principal.
    assert_eq!(runner.set_sender(STAKER_ADDR_1).rtd_balance(), (550 + 150) * MIST_PER_RTD);
    assert_eq!(runner.set_sender(STAKER_ADDR_2).rtd_balance(), 100 * MIST_PER_RTD);

    runner.finish();
}

#[test]
fun rewards_slashing_with_storage_fund() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1000)
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    let validators = runner.genesis_validator_addresses();

    let opts = runner.advance_epoch_opts().storage_charge(300);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // Add a few stakes.
    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_3, 100);
    runner.set_sender(STAKER_ADDR_2).stake_with(VALIDATOR_ADDR_4, 100);
    runner.advance_epoch(option::none()).destroy_for_testing();

    // validator_4 is reported by 3 other validators, so 75% of total stake.
    vector[VALIDATOR_ADDR_1, VALIDATOR_ADDR_2, VALIDATOR_ADDR_3].do!(|validator_address| {
        runner.set_sender(validator_address).report_validator(VALIDATOR_ADDR_4);
    });

    // 1000 RTD of storage rewards, 1500 RTD of computation rewards, 50% slashing threshold
    // and 20% slashing rate
    let opts = runner
        .advance_epoch_opts()
        .storage_charge(1000)
        .computation_charge(1500)
        .reward_slashing_rate(2000);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // Each unslashed validator staking pool gets 300 RTD of computation rewards + 75 RTD of storage fund rewards +
    // 20 RTD (1/3) of validator 4's slashed computation reward and 5 RTD (1/3) of validator 4's slashed
    // storage fund reward, so in total it gets 400 RTD of rewards.
    // Validator 3 has a delegator with her so she gets 320 * 3/4 + 75 + 5 = 320 RTD of rewards.
    // Validator 4's should get 300 * 4/5 * (1 - 20%) = 192 in computation rewards and 75 * (1 - 20%) = 60 in storage rewards.
    assert_stake_rewards_for_addresses(
        &mut runner,
        validators,
        vector[500 * MIST_PER_RTD, 600 * MIST_PER_RTD, 620 * MIST_PER_RTD, 652 * MIST_PER_RTD],
    );

    // Unstake so we can check the stake rewards as well.
    runner.set_sender(STAKER_ADDR_1).unstake(0);
    runner.set_sender(STAKER_ADDR_2).unstake(0);

    // Staker 1 gets 320 * 1/4 = 80 RTD of rewards.
    assert_eq!(runner.set_sender(STAKER_ADDR_1).rtd_balance(), (100 + 80) * MIST_PER_RTD);
    // Staker 2 gets 300 * 1/5 * (1 - 20%) = 48 RTD of rewards.
    assert_eq!(runner.set_sender(STAKER_ADDR_2).rtd_balance(), (100 + 48) * MIST_PER_RTD);

    runner.finish();
}

#[test]
// This test is to make sure that if everyone is slashed, our protocol works as expected without aborting
// and all rewards go to the storage fund.
fun everyone_slashed() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1000)
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    let validators = runner.genesis_validator_addresses();

    // Each validator reports all other validators but themselves
    validators.do!(|validator_address| {
        validators.do!(|other_validator_address| {
            if (other_validator_address != validator_address) {
                runner.set_sender(validator_address).report_validator(other_validator_address);
            }
        });
    });

    let opts = runner
        .advance_epoch_opts()
        .storage_charge(1000)
        .computation_charge(3000)
        .reward_slashing_rate(10_000);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // All validators should have 0 rewards added so their stake stays the same.
    assert_stake_rewards_for_addresses(
        &mut runner,
        validators,
        vector[100 * MIST_PER_RTD, 200 * MIST_PER_RTD, 300 * MIST_PER_RTD, 400 * MIST_PER_RTD],
    );

    runner.system_tx!(|system, _| {
        // Storage fund balance should increase by 4000 RTD.
        assert_eq!(system.get_storage_fund_total_balance(), 4000 * MIST_PER_RTD);
        // The entire 1000 RTD of storage rewards should go to the object rebate portion of the storage fund.
        assert_eq!(system.get_storage_fund_object_rebates(), 1000 * MIST_PER_RTD);
    });

    runner.finish();
}

#[test]
fun mul_rewards_withdraws_at_same_epoch() {
    let mut runner = test_runner::new()
        .rtd_supply_amount(1000)
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200).rtd_address(VALIDATOR_ADDR_2),
            validator_builder::new().initial_stake(300).rtd_address(VALIDATOR_ADDR_3),
            validator_builder::new().initial_stake(400).rtd_address(VALIDATOR_ADDR_4),
        ])
        .build();

    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 220);

    let opts = runner.advance_epoch_opts().computation_charge(40);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    runner.set_sender(STAKER_ADDR_2).stake_with(VALIDATOR_ADDR_1, 480);

    // Staker 1 gets 2/3 * 1/4 * 120 = 20 RTD here.
    let opts = runner.advance_epoch_opts().computation_charge(120);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 130);
    runner.set_sender(STAKER_ADDR_3).stake_with(VALIDATOR_ADDR_1, 390);

    // Staker 1 gets 20 RTD here and staker 2 gets 40 RTD here.
    let opts = runner.advance_epoch_opts().computation_charge(280);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    runner.set_sender(STAKER_ADDR_3).stake_with(VALIDATOR_ADDR_1, 280);
    runner.set_sender(STAKER_ADDR_4).stake_with(VALIDATOR_ADDR_1, 1400);

    // Staker 1 gets 30 RTD, staker 2 gets 40 RTD and staker 3 gets 30 RTD.
    let opts = runner.advance_epoch_opts().computation_charge(440);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // Check that we have the right amount of RTD in the staking pool.
    runner.system_tx!(|system, _| {
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_1), 140 * 23 * MIST_PER_RTD);
    });

    // Withdraw all stakes at once.
    runner.set_sender(STAKER_ADDR_1).unstake(0);
    runner.set_sender(STAKER_ADDR_1).unstake(0);
    runner.set_sender(STAKER_ADDR_2).unstake(0);
    runner.set_sender(STAKER_ADDR_3).unstake(0);
    runner.set_sender(STAKER_ADDR_3).unstake(0);
    runner.set_sender(STAKER_ADDR_4).unstake(0);

    // staker 1's first stake was active for 3 epochs so got 20 * 3 = 60 RTD of rewards
    // and her second stake was active for only one epoch and got 10 RTD of rewards.
    assert_eq!(
        runner.set_sender(STAKER_ADDR_1).rtd_balance(),
        (220 + 130 + 20 * 3 + 10) * MIST_PER_RTD,
    );
    // staker 2's stake was active for 2 epochs so got 40 * 2 = 80 RTD of rewards
    assert_eq!(runner.set_sender(STAKER_ADDR_2).rtd_balance(), (480 + 40 * 2) * MIST_PER_RTD);
    // staker 3's first stake was active for 1 epoch and got 30 RTD of rewards
    // and her second stake didn't get any rewards.
    assert_eq!(runner.set_sender(STAKER_ADDR_3).rtd_balance(), (390 + 280 + 30) * MIST_PER_RTD);
    // staker 4 joined and left in an epoch where no rewards were earned so she got no rewards.
    assert_eq!(runner.set_sender(STAKER_ADDR_4).rtd_balance(), 1400 * MIST_PER_RTD);

    runner.advance_epoch(option::none()).destroy_for_testing();

    // Since all the stakes are gone the pool is empty except for the validator's original stake.
    runner.system_tx!(|system, _| {
        assert_eq!(system.validator_stake_amount(VALIDATOR_ADDR_1), 140 * MIST_PER_RTD);
    });

    runner.finish();
}

#[test]
fun uncapped_rewards() {
    let num_validators = 20;
    let validators = vector::tabulate!(num_validators, |i| {
        validator_builder::new()
            .initial_stake(481 + i * 2)
            .rtd_address(address::from_u256(i as u256))
    });

    let mut runner = test_runner::new().rtd_supply_amount(1000).validators(validators).build();

    // Each validator's stake gets doubled.
    let opts = runner.advance_epoch_opts().computation_charge(10_000);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // Check that each validator has the correct amount of RTD in their stake pool.
    runner.system_tx!(|system, _| {
        num_validators.do!(|i| {
            let addr = address::from_u256(i as u256);
            assert_eq!(system.validator_stake_amount(addr), (962 + i * 4) * MIST_PER_RTD);
        });
    });

    runner.finish();
}

#[test]
// Safe Mode Testing Scenario:
// - mimic state during epoch 562, if we're in safe mode since the 560 -> 561 epoch change
// - advance epoch to 563
// - ensure that all previous epochs are distributed
// - ensure that next epoch change only distributes one epoch's worth
fun stake_subsidy_with_safe_mode_epoch_562_to_563() {
    let epoch_duration = 1000;
    let epoch_start_time = 100000000000;
    // distribution counter is lagging behind by 20 epochs on mainnet and testnet
    let start_distribution_counter = 540;
    let mut runner = test_runner::new()
        .validators_count(4)
        .validators_initial_stake(1_000_000_000)
        .rtd_supply_amount(1_000_000_000)
        .stake_distribution_counter(start_distribution_counter)
        .epoch_duration(epoch_duration)
        .start_epoch(562)
        .build();

    // mimic state during epoch 562, if we're in safe mode since the 560 -> 561 epoch change
    runner.system_tx!(|system, ctx| {
        assert_eq!(ctx.epoch(), 562);
        assert_eq!(system.epoch(), 562);
        assert_eq!(system.get_stake_subsidy_distribution_counter(), start_distribution_counter);
    });

    // perform advance epoch
    let opts = runner.advance_epoch_opts().protocol_version(65).epoch_start_time(epoch_start_time);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // should distribute 3 epochs worth of subsidies: 560, 561, 562
    runner.system_tx!(|system, ctx| {
        assert_eq!(ctx.epoch(), 563);
        assert_eq!(system.epoch(), 563);
        assert_eq!(system.get_stake_subsidy_distribution_counter(), start_distribution_counter + 3);
        check_distribution_counter_invariant(system, ctx);
    });

    // ensure that next epoch change only distributes one epoch's worth
    let opts = runner
        .advance_epoch_opts()
        .protocol_version(65)
        .epoch_start_time(epoch_start_time + epoch_duration);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // should distribute 1 epoch's worth of subsidies: 563 only
    runner.system_tx!(|system, ctx| {
        assert_eq!(ctx.epoch(), 564);
        assert_eq!(system.epoch(), 564);
        assert_eq!(system.get_stake_subsidy_distribution_counter(), start_distribution_counter + 4);
        check_distribution_counter_invariant(system, ctx);
    });

    runner.finish();
}

#[test]
// Safe Mode Testing Scenario:
// - mimic state during epoch 563, if we're in safe mode since the 560 -> 561 epoch change
// - advance epoch to 564
// - ensure that all previous epochs are distributed
// - ensure that next epoch change only distributes one epoch's worth
fun stake_subsidy_with_safe_mode_epoch_563_to_564() {
    let epoch_duration = 1000;
    let epoch_start_time = 100000000000;
    // distribution counter is lagging behind by 20 epochs on mainnet and testnet
    let start_distribution_counter = 540;
    let mut runner = test_runner::new()
        .validators_count(4)
        .validators_initial_stake(1_000_000_000)
        .rtd_supply_amount(1_000_000_000)
        .stake_distribution_counter(start_distribution_counter)
        .epoch_duration(epoch_duration)
        .start_epoch(563)
        .build();

    // check initial state
    runner.system_tx!(|system, ctx| {
        assert_eq!(ctx.epoch(), 563);
        assert_eq!(system.epoch(), 563);
        assert_eq!(system.get_stake_subsidy_distribution_counter(), start_distribution_counter);
    });

    // perform advance epoch
    let opts = runner.advance_epoch_opts().protocol_version(65).epoch_start_time(epoch_start_time);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // should distribute 4 epochs worth of subsidies: 560, 561, 562, 563
    runner.system_tx!(|system, ctx| {
        assert_eq!(ctx.epoch(), 564);
        assert_eq!(system.epoch(), 564);
        assert_eq!(system.get_stake_subsidy_distribution_counter(), start_distribution_counter + 4);
        check_distribution_counter_invariant(system, ctx);
    });

    // ensure that next epoch change only distributes one epoch's worth
    let opts = runner
        .advance_epoch_opts()
        .protocol_version(65)
        .epoch_start_time(epoch_start_time + epoch_duration);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // should distribute 1 epoch's worth of subsidies: 564 only
    runner.system_tx!(|system, ctx| {
        assert_eq!(ctx.epoch(), 565);
        assert_eq!(system.epoch(), 565);
        assert_eq!(system.get_stake_subsidy_distribution_counter(), start_distribution_counter + 5);
        check_distribution_counter_invariant(system, ctx);
    });

    runner.finish();
}

#[test]
// TODO: come back to me once safe mode emulation is implemented
fun stake_subsidy_with_safe_mode_testnet() {
    let mut runner = test_runner::new()
        .validators_count(4)
        .validators_initial_stake(1_000_000_000)
        .rtd_supply_amount(1_000_000_000)
        .stake_distribution_counter(540)
        .epoch_duration(1000)
        .start_epoch(540)
        .build();

    // check initial state
    runner.system_tx!(|system, ctx| {
        assert_eq!(ctx.epoch(), 540);
        assert_eq!(system.epoch(), 540);
        assert_eq!(system.get_stake_subsidy_distribution_counter(), 540);
    });

    // perform advance epoch
    let opts = runner.advance_epoch_opts().protocol_version(65).epoch_start_time(99_9999_999);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // check that the distribution counter is incremented
    runner.system_tx!(|system, ctx| {
        assert_eq!(ctx.epoch(), 541);
        assert_eq!(system.epoch(), 541);
        assert_eq!(system.get_stake_subsidy_distribution_counter(), 541);
    });

    runner.finish();
}

#[test]
// This test triggers both rtd balance and pool token to fall short but no underflow happens.
fun process_pending_stake_withdraw_no_underflow_in_safe_mode_1() {
    // 4 validators, each with 100, 200, 300, 400 RTD
    // safe mode epoch 1
    let mut runner = test_runner::new()
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200),
            validator_builder::new().initial_stake(300),
            validator_builder::new().initial_stake(400),
        ])
        .start_epoch(1) // safe mode epoch 1
        .build();

    // check initial stake of validator 1
    runner.system_tx!(|system, _| {
        let pool = system.active_validator_by_address(VALIDATOR_ADDR_1).get_staking_pool_ref();

        assert_eq!(pool.rtd_balance(), 100 * MIST_PER_RTD);
        assert_eq!(pool.pool_token_balance(), 100 * MIST_PER_RTD);
    });

    // staker 1 stakes 101 rtd in safe mode
    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 101);

    // check stake of validator 1 including pending values
    runner.system_tx!(|system, _| {
        let pool = system.active_validator_by_address(VALIDATOR_ADDR_1).get_staking_pool_ref();

        assert_eq!(pool.rtd_balance(), 100 * MIST_PER_RTD);
        assert_eq!(pool.pool_token_balance(), 100 * MIST_PER_RTD);
        assert_eq!(pool.pending_stake_amount(), 101 * MIST_PER_RTD);
    });

    // still safe mode, now epoch 2
    runner.advance_epoch_safe_mode();

    // staker 1 unstakes in safe mode
    runner.set_sender(STAKER_ADDR_1).unstake(0);

    // state invariants check
    runner.system_tx!(|system, _| {
        let pool = system.active_validator_by_address(VALIDATOR_ADDR_1).get_staking_pool_ref();
        assert_eq!(pool.rtd_balance(), 100 * MIST_PER_RTD);
        assert_eq!(pool.pool_token_balance(), 100 * MIST_PER_RTD);
        assert_eq!(pool.pending_stake_amount(), 101 * MIST_PER_RTD);
        assert_eq!(pool.pending_stake_withdraw_amount(), 101 * MIST_PER_RTD);
        assert_eq!(pool.pending_pool_token_withdraw_amount(), 101 * MIST_PER_RTD);
    });

    // epoch 3: exiting safe mode, no underflow
    let opts = runner.advance_epoch_opts().protocol_version(65).epoch_start_time(99_9999_999);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // store Pool ID to query inactive validator later
    let pool_id: ID;

    runner.system_tx!(|system, _| {
        let pool = system.active_validator_by_address(VALIDATOR_ADDR_1).get_staking_pool_ref();
        pool_id = object::id(pool);

        assert_eq!(pool.pending_stake_amount(), 0 * MIST_PER_RTD);
        assert_eq!(pool.pending_stake_withdraw_amount(), 0 * MIST_PER_RTD);
        assert_eq!(pool.pending_pool_token_withdraw_amount(), 0 * MIST_PER_RTD);
        assert_eq!(pool.rtd_balance(), 100 * MIST_PER_RTD);
        assert_eq!(pool.pool_token_balance(), 100 * MIST_PER_RTD);
    });

    // validator 1 withdraws its stake and gets back 100 RTD
    runner.set_sender(VALIDATOR_ADDR_1).unstake(0);

    // advance epoch to 4, no safe mode, no rewards
    runner.advance_epoch(option::none()).destroy_for_testing();

    // enforce invariant: rtd_balance = 0
    runner.system_tx!(|system, _| {
        let pool = system
            .validators()
            .inactive_validator_by_pool_id(pool_id)
            .get_staking_pool_ref();

        assert_eq!(pool.rtd_balance(), 0 * MIST_PER_RTD);
        assert_eq!(pool.pool_token_balance(), 0 * MIST_PER_RTD);
    });

    runner.finish();
}

#[test]
// This test triggers pool token to fall short but no underflow happens.
fun process_pending_stake_withdraw_no_underflow_in_safe_mode_2() {
    // 4 validators, each with 100, 200, 300, 400 RTD
    // safe mode epoch 1
    let mut runner = test_runner::new()
        .validators(vector[
            validator_builder::new().initial_stake(100).rtd_address(VALIDATOR_ADDR_1),
            validator_builder::new().initial_stake(200),
            validator_builder::new().initial_stake(300),
            validator_builder::new().initial_stake(400),
        ])
        .build();

    // check initial stake of validator 1
    runner.system_tx!(|system, _| {
        let pool = system.active_validator_by_address(VALIDATOR_ADDR_1).get_staking_pool_ref();

        assert_eq!(pool.rtd_balance(), 100 * MIST_PER_RTD);
        assert_eq!(pool.pool_token_balance(), 100 * MIST_PER_RTD);
    });

    // Epoch 1:
    let opts = runner
        .advance_epoch_opts()
        .protocol_version(65)
        .computation_charge(100) // 100 RTD computation charge
        .epoch_start_time(99_9999_999);

    runner.advance_epoch(option::some(opts)).destroy_for_testing();
    runner.system_tx!(|system, _| {
        assert_eq!(system.epoch(), 1);
        let pool = system.active_validator_by_address(VALIDATOR_ADDR_1).get_staking_pool_ref();

        assert_eq!(pool.rtd_balance(), 125 * MIST_PER_RTD);
        assert_eq!(pool.pool_token_balance(), 100 * MIST_PER_RTD);
    });

    // Epoch 2: entering safe mode
    runner.advance_epoch_safe_mode();

    // staker 1 stakes 100 rtd in epoch 2
    runner.set_sender(STAKER_ADDR_1).stake_with(VALIDATOR_ADDR_1, 100);

    // Epoch 3: still in safe mode
    runner.advance_epoch_safe_mode();

    // Epoch 4: still in safe mode
    runner.advance_epoch_safe_mode();

    // Epoch 5: now out of safe mode
    let opts = runner
        .advance_epoch_opts()
        .protocol_version(65)
        .computation_charge(100) // 100 RTD computation charge
        .epoch_start_time(99_9999_999);

    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    runner.system_tx!(|system, _| {
        let pool = system.active_validator_by_address(VALIDATOR_ADDR_1).get_staking_pool_ref();
        let exchange_rate = pool.exchange_rates()[5];
        assert_eq!(exchange_rate.rtd_amount(), 250 * MIST_PER_RTD);

        // old pool token balance / old pool rtd balance * (pool rtd balance + pending stake)
        // 100 / 150 * (150 + 100) = 166666666666
        assert_eq!(exchange_rate.pool_token_amount(), 166666666666);
        assert_eq!(pool.rtd_balance(), 250 * MIST_PER_RTD);
        assert_eq!(pool.pool_token_balance(), 166666666666);
    });

    // staker 1 unstakes
    runner.set_sender(STAKER_ADDR_1).owned_tx!<rtd_system::staking_pool::StakedRtd>(|stake| {
        assert_eq!(stake.stake_activation_epoch(), 3);
        runner.system_tx!(|system, ctx| {
            system.request_withdraw_stake(stake, ctx);

            // Pool's exchange rate for epoch 2 is missing because of safe mode
            // So it fallbacks to epoch 1's exchange rate: PoolTokenExchangeRate {
            //   rtd_amount: 125000000000,
            //   pool_token_amount: 100000000000
            // }
            // pending pool token to withdraw: 100 (principal) / 125 * 100 = 80
            let pool = system.active_validator_by_address(VALIDATOR_ADDR_1).get_staking_pool_ref();

            assert_eq!(pool.pending_pool_token_withdraw_amount(), 80 * MIST_PER_RTD);
            // exchange rate for epoch 5: 1666...6: 250
            // total withdraw: 80 * 250 / 166...6 = 120 rtd
            assert_eq!(pool.pending_stake_withdraw_amount(), 120 * MIST_PER_RTD); // 100 principal + 20 rewards
        });
    });

    // Epoch 6:
    let opts = runner
        .advance_epoch_opts()
        .protocol_version(65)
        .computation_charge(100) // 100 RTD computation charge
        .epoch_start_time(99_9999_999);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // Validator unstakes
    runner.set_sender(VALIDATOR_ADDR_1).unstake(0);

    let pool_id: ID;

    runner.system_tx!(|system, _| {
        let pool = system.active_validator_by_address(VALIDATOR_ADDR_1).get_staking_pool_ref();
        pool_id = object::id(pool);

        assert_eq!(pool.rtd_balance(), 155 * MIST_PER_RTD);
        assert_eq!(pool.pending_stake_withdraw_amount(), 155 * MIST_PER_RTD);

        // epoch 0's exchange rate: PoolTokenExchangeRate {
        //   rtd_amount: 100000000000,
        //   pool_token_amount: 100000000000
        // }
        // pending pool token to withdraw: 100 (principal) / 100 * 100 = 100
        // exchange rate for epoch 6: 155000000000: 86666666666
        // total withdraw: min(100 * 155000000000 / 86666666666, pool.rtd_balance()) = 155000000000
        assert_eq!(pool.pending_stake_withdraw_amount(), 155 * MIST_PER_RTD); // 100 principal + 55 rewards
        let exchange_rates = pool.exchange_rates();
        let exchange_rate_epoch_0 = exchange_rates.borrow(0);
        assert_eq!(exchange_rate_epoch_0.rtd_amount(), 0);
        assert_eq!(exchange_rate_epoch_0.pool_token_amount(), 0);
        let exchange_rate_epoch_1 = exchange_rates.borrow(1);
        assert_eq!(exchange_rate_epoch_1.rtd_amount(), 125000000000);
        assert_eq!(exchange_rate_epoch_1.pool_token_amount(), 100000000000);
        assert!(!exchange_rates.contains(2));
        assert!(!exchange_rates.contains(3));
        assert!(!exchange_rates.contains(4));
        let exchange_rate_epoch_5 = exchange_rates.borrow(5);
        assert_eq!(exchange_rate_epoch_5.rtd_amount(), 250000000000);
        assert_eq!(exchange_rate_epoch_5.pool_token_amount(), 166666666666);
        let exchange_rate_epoch_6 = exchange_rates.borrow(6);
        assert_eq!(exchange_rate_epoch_6.rtd_amount(), 155000000000);
        assert_eq!(exchange_rate_epoch_6.pool_token_amount(), 86666666666);

        // insufficient pool token balance
        assert_eq!(pool.rtd_balance(), 155000000000);
        assert_eq!(pool.pending_stake_withdraw_amount(), 155000000000);
        assert_eq!(pool.pool_token_balance(), 86666666666);
        assert_eq!(pool.pending_pool_token_withdraw_amount(), 100000000000);

        assert!(pool.pool_token_balance() < pool.pending_pool_token_withdraw_amount());
    });

    // Epoch 7:
    // No underflow should happen
    let opts = runner.advance_epoch_opts().protocol_version(65).epoch_start_time(99_9999_999);
    runner.advance_epoch(option::some(opts)).destroy_for_testing();

    // Check that the validator is inactive and has no pending stake or pool token to withdraw
    runner.system_tx!(|system, _| {
        assert!(system.validators().is_inactive_validator(pool_id));

        let validator = system.validators().inactive_validator_by_pool_id(pool_id);

        assert_eq!(validator.pending_stake_amount(), 0);
        assert_eq!(validator.pending_stake_withdraw_amount(), 0);
        assert_eq!(validator.get_staking_pool_ref().pending_pool_token_withdraw_amount(), 0);
        assert_eq!(validator.get_staking_pool_ref().rtd_balance(), 0);
        assert_eq!(validator.get_staking_pool_ref().pool_token_balance(), 0);
    });

    runner.finish();
}

// TODO: potentially remove and inline this function.
fun check_distribution_counter_invariant(system: &mut RtdSystemState, ctx: &TxContext) {
    assert_eq!(ctx.epoch(), system.epoch());
    // first subsidy distribution was at epoch 20, so counter should always be ahead by 20
    assert_eq!(system.get_stake_subsidy_distribution_counter() + 20, ctx.epoch());
}

/// Utility function to assert that the stake rewards for a list of addresses are as expected.
fun assert_stake_rewards_for_addresses(
    runner: &mut test_runner::TestRunner,
    validator_addresses: vector<address>,
    expected_amounts: vector<u64>,
) {
    validator_addresses.zip_do!(expected_amounts, |validator_address, expected_amount| {
        let sum_rewards = runner.set_sender(validator_address).staking_rewards_balance();
        assert_eq!(sum_rewards, expected_amount);
    });
}
