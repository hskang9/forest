// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use address::Address;
use clock::ChainEpoch;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use runtime::{ActorCode, Runtime};
use std::collections::HashMap;
use vm::{
    ExitCode, InvocOutput, MethodNum, Serialized, SysCode, TokenAmount, METHOD_CONSTRUCTOR,
    METHOD_PLACEHOLDER,
};

pub struct Reward {
    pub start_epoch: ChainEpoch,
    pub value: TokenAmount,
    pub release_rate: TokenAmount,
    pub amount_withdrawn: TokenAmount,
}

/// RewardActorState has no internal state
pub struct RewardActorState {
    pub reward_map: HashMap<Address, Vec<Reward>>,
}

impl RewardActorState {
    pub fn withdraw_reward<RT: Runtime>(_rt: &RT, _owner: Address) -> TokenAmount {
        // TODO
        TokenAmount::new(0)
    }
}

#[derive(FromPrimitive)]
pub enum RewardMethod {
    Constructor = METHOD_CONSTRUCTOR,
    MintReward = METHOD_PLACEHOLDER,
    WithdrawReward = METHOD_PLACEHOLDER + 1,
}

impl RewardMethod {
    /// from_method_num converts a method number into an RewardMethod enum
    fn from_method_num(m: MethodNum) -> Option<RewardMethod> {
        FromPrimitive::from_u64(u64::from(m))
    }
}

#[derive(Clone)]
pub struct RewardActorCode;
impl RewardActorCode {
    /// Constructor for Reward actor
    fn constructor<RT: Runtime>(_rt: &RT) -> InvocOutput {
        // TODO
        unimplemented!();
    }
    /// Mints a reward and puts into state reward map
    fn mint_reward<RT: Runtime>(_rt: &RT) -> InvocOutput {
        // TODO
        unimplemented!();
    }
    /// Withdraw available funds from reward map
    fn withdraw_reward<RT: Runtime>(_rt: &RT) -> InvocOutput {
        // TODO
        unimplemented!();
    }
}

impl ActorCode for RewardActorCode {
    fn invoke_method<RT: Runtime>(
        &self,
        rt: &RT,
        method: MethodNum,
        _params: &Serialized,
    ) -> InvocOutput {
        match RewardMethod::from_method_num(method) {
            // TODO determine parameters for each method on finished spec
            Some(RewardMethod::Constructor) => Self::constructor(rt),
            Some(RewardMethod::MintReward) => Self::mint_reward(rt),
            Some(RewardMethod::WithdrawReward) => Self::withdraw_reward(rt),
            _ => {
                rt.abort(
                    ExitCode::SystemErrorCode(SysCode::InvalidMethod),
                    "Invalid method",
                );
                unreachable!();
            }
        }
    }
}
