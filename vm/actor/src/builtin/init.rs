// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{ActorID, CodeID};
use vm::{
    ExitCode, InvocOutput, MethodNum, Serialized, SysCode, METHOD_CONSTRUCTOR, METHOD_PLACEHOLDER,
};

use address::Address;
use encoding::Cbor;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use runtime::{ActorCode, Runtime};
use std::collections::HashMap;

/// InitActorState is reponsible for creating
#[derive(Default)]
pub struct InitActorState {
    // TODO possibly switch this to a hamt to be able to dump the data and save as Cid
    _address_map: HashMap<Address, ActorID>,
    next_id: ActorID,
}

impl InitActorState {
    /// Assigns next available ID and incremenets the next_id value from state
    pub fn assign_next_id(&mut self) -> ActorID {
        let next = self.next_id;
        self.next_id.0 += 1;
        next
    }
}

#[derive(FromPrimitive)]
pub enum InitMethod {
    Constructor = METHOD_CONSTRUCTOR,
    Exec = METHOD_PLACEHOLDER,
    GetActorIDForAddress = METHOD_PLACEHOLDER + 1,
}

impl InitMethod {
    /// from_method_num converts a method number into an InitMethod enum
    fn from_method_num(m: MethodNum) -> Option<InitMethod> {
        FromPrimitive::from_u64(u64::from(m))
    }
}

pub struct InitActorCode;
impl InitActorCode {
    fn constructor<RT: Runtime>(rt: &RT) -> InvocOutput {
        // Acquire state
        // Update actor substate

        rt.success_return()
    }
    fn exec<RT: Runtime>(rt: &RT, _code: CodeID, _params: &Serialized) -> InvocOutput {
        // TODO
        let addr = Address::new_id(0).unwrap();
        rt.value_return(addr.marshal_cbor().unwrap())
    }
    fn get_actor_id_for_address<RT: Runtime>(rt: &RT, _address: Address) -> InvocOutput {
        // TODO
        rt.value_return(ActorID(0).marshal_cbor().unwrap())
    }
}

impl ActorCode for InitActorCode {
    fn invoke_method<RT: Runtime>(
        &self,
        rt: &RT,
        method: MethodNum,
        params: &Serialized,
    ) -> InvocOutput {
        // Create mutable copy of params for usage in functions
        let params: &mut Serialized = &mut params.clone();
        match InitMethod::from_method_num(method) {
            Some(InitMethod::Constructor) => {
                // TODO unfinished spec

                Self::constructor(rt)
            }
            Some(InitMethod::Exec) => {
                // TODO deserialize CodeID on finished spec
                Self::exec(rt, CodeID::Init, params)
            }
            Some(InitMethod::GetActorIDForAddress) => {
                // Unmarshall address parameter
                // TODO unfinished spec

                // Errors checked, get actor by address
                Self::get_actor_id_for_address(rt, Address::default())
            }
            _ => {
                // Method number does not match available, abort in runtime
                rt.abort(
                    ExitCode::SystemErrorCode(SysCode::InvalidMethod),
                    "Invalid method",
                );
                unreachable!();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn assign_id() {
        let mut actor_s = InitActorState::default();
        assert_eq!(actor_s.assign_next_id().0, 0);
        assert_eq!(actor_s.assign_next_id().0, 1);
        assert_eq!(actor_s.assign_next_id().0, 2);
    }
}
