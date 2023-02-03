use revm::{
    Database,
    db::{CacheDB, EmptyDB},
    primitives::{AccountInfo, Bytecode, B160, U256, ExecutionResult, TransactTo, ruint::Uint},
    EVM,
};
use bytes::Bytes;
use std::str::FromStr;
use revm::primitives::result::EVMError;

/// Uniswap V3 factory address.
const FACTORY: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";

#[derive(Default)]
pub struct ExecutionManager {
    pub evm: EVM<CacheDB<EmptyDB>>,
}

impl ExecutionManager {
    /// Public constructor function to instantiate an `ExecutionManager`.
    pub fn new() -> Self {
        let mut evm = EVM::new();
        let mut db = CacheDB::new(EmptyDB {});

        evm.database(db);

        Self { evm }
    }

    /// Deploy a contract to the Execution instance.
    pub fn deploy_contract(&mut self, bytecode: Bytecode) {
        let factory_info = AccountInfo::new(U256::from(0), 0, bytecode);

        self.evm.db().unwrap().insert_account_info(B160::from_str(FACTORY).unwrap(), factory_info);
    }

    /// Execute a transaction.
    pub fn execute(
        &mut self, 
        caller: B160,
        data: Bytes,
        transact_to: TransactTo,
        value: U256,
    ) -> ExecutionResult {
        self.evm.env.tx.caller = caller;
        self.evm.env.tx.transact_to = transact_to;
        self.evm.env.tx.data = data;
        self.evm.env.tx.value = value;

        match self.evm.transact_commit() {
            Ok(val) => return val,
            // URGENT: change this to a custom error
            Err(_) => panic!("failed"),
        };
    }

    /// Give an address a specified amount of ether.
    pub fn deal(&mut self, address: B160, amount: U256) {
        let account = self.evm.db().unwrap().load_account(address).unwrap();

        account.info.balance = amount;
    }
}