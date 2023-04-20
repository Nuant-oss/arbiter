#![warn(missing_docs)]
//! Describes the most basic type of user agent.

use crossbeam_channel::Receiver;
use revm::primitives::{AccountInfo, Address, Log, B160, U256};

use crate::agent::{Agent, TransactSettings};

/// A user is an agent that can interact with the simulation environment generically.
pub struct User {
    /// Name of the agent.
    pub name: String,
    /// Public address of the simulation manager.
    pub address: B160,
    /// [`revm::primitives`] account of the simulation manager.
    pub account_info: AccountInfo,
    /// Contains the default transaction options for revm such as gas limit and gas price.
    pub transact_settings: TransactSettings,
    /// The [`crossbeam_channel::Receiver`] for the events are sent down from [`SimulationEnvironment`]'s dispatch.
    pub event_receiver: Receiver<Vec<Log>>,
}

impl Agent for User {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn address(&self) -> Address {
        self.address
    }
    fn transact_settings(&self) -> &TransactSettings {
        &self.transact_settings
    }
    fn receiver(&self) -> Receiver<Vec<Log>> {
        self.event_receiver.clone()
    }
    fn filter_events(&self, _logs: Vec<Log>) -> Vec<Log> {
        todo!();
    }
}

impl User {
    /// Constructor function to instantiate a user agent.
    pub(crate) fn new(name: String, address: B160, event_receiver: Receiver<Vec<Log>>) -> Self {
        Self {
            name,
            address,
            account_info: AccountInfo::default(),
            transact_settings: TransactSettings {
                gas_limit: u64::MAX,   // TODO: Users should have a gas limit.
                gas_price: U256::ZERO, // TODO: Users should have an associated gas price.
            },
            event_receiver,
        }
    }
}

#[cfg(test)]
mod tests {
    fn user_activation_test() {
        todo!();
    }
}
