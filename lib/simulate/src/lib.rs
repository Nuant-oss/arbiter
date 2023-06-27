#![warn(missing_docs)]
#![warn(unsafe_code)]
//! Lib crate for describing simulations.

pub mod agent;
pub mod environment;
pub mod exchange;
pub mod historic;
pub mod manager;
pub mod stochastic;
pub mod utils;

#[cfg(test)]
mod tests {
    use std::error::Error;

    use bindings::{arbiter_token, writer};
    use ethers::{abi::Tokenize, prelude::U256, types::I256};
    use revm::primitives::B160;

    use crate::{
        agent::{user::User, Agent, AgentType},
        environment::contract::SimulationContract,
        manager::SimulationManager,
        utils::{recast_address, unpack_execution},
    };

    #[test]
    /// Test that the writer contract can echo a string.
    /// The writer contract takes in no constructor args.
    fn string_write() -> Result<(), Box<dyn Error>> {
        // Set up the execution manager and a user address.
        let manager = SimulationManager::default();
        let admin = &manager.agents["admin"];

        // Get bytecode and abi for the writer contract.
        let writer =
            SimulationContract::new(writer::WRITER_ABI.clone(), writer::WRITER_BYTECODE.clone());

        // Deploy the writer contract.
        let (writer, execution_result) = admin.deploy(writer, ().into_tokens())?;
        assert!(execution_result.is_success());

        // Call the 'echoString' function.
        let test_string = "Hello, world!".to_string();
        let execution_result =
            admin.call(&writer, "echoString", test_string.clone().into_tokens())?;
        let value = unpack_execution(execution_result)?;

        let response: String = writer.decode_output("echoString", value)?;

        println!("Writing Response: {response:#?}");
        assert_eq!(response, test_string);
        Ok(())
    }

    #[test]
    /// Test to see that we can mint tokens to a user.
    fn token_mint() -> Result<(), Box<dyn Error>> {
        // Create a `SimulationManager` where we can run simulations.
        // This will also create an EVM instance associated to the manager.
        let mut manager = SimulationManager::default();

        // Get the relevant users.
        let user_name = "alice";
        let user_address = B160::from_low_u64_be(2);
        let alice = User::new(user_name, None);
        manager.activate_agent(AgentType::User(alice), user_address)?;
        let admin = &manager.agents["admin"];
        let alice = &manager.agents[user_name];

        // Get a SimulationContract for the Arbiter Token ERC-20 instance from the ABI and bytecode.
        let arbiter_token = SimulationContract::new(
            arbiter_token::ARBITERTOKEN_ABI.clone(),
            arbiter_token::ARBITERTOKEN_BYTECODE.clone(),
        );

        // Choose name and symbol and combine into the constructor args required by ERC-20 contracts.
        let name = "ArbiterToken";
        let symbol = "ARBT";
        let args = (name.to_string(), symbol.to_string(), 18_u8);

        // Call the contract deployer and receive a IsDeployed version of SimulationContract that now has an address.
        let (arbiter_token, execution_result) = admin.deploy(arbiter_token, args.into_tokens())?;
        assert!(execution_result.is_success());
        println!("Arbiter Token deployed at: {}", arbiter_token.address);

        // Execute the call to retrieve the token name as a test.
        let execution_result = admin.call(&arbiter_token, "name", ().into_tokens())?;
        let value = unpack_execution(execution_result)?;

        let response: String = arbiter_token.decode_output("name", value)?;
        assert_eq!(response, name); // Quick check that the name is correct.

        // Allocating new tokens to user by calling Arbiter Token's ERC20 'mint' instance.
        let mint_amount = U256::from(1000);

        // Set up the args for the 'mint' function.
        let input_arguments = (recast_address(alice.address()), mint_amount);

        // Call the 'mint' function.
        let execution_result = admin.call(&arbiter_token, "mint", input_arguments.into_tokens());
        println!("Mint execution result: {:#?}", execution_result);

        // Call the 'balanceOf' function.
        let execution_result = admin.call(
            &arbiter_token,
            "balanceOf",
            recast_address(alice.address()).into_tokens(),
        )?;
        let value = unpack_execution(execution_result)?;

        let response: U256 = arbiter_token.decode_output("balanceOf", value)?;

        assert_eq!(response, mint_amount); // Check that the value minted is correct.
        Ok(())
    }

    #[test]
    fn auto_deploy() {
        let manager = SimulationManager::new();
        assert!(manager.deployed_contracts.get("arbiter_math").is_some());
    }

    #[test]
    fn arbiter_math() -> Result<(), Box<dyn Error>> {
        // Create a `SimulationManager` where we can run simulations.
        // This will also create an EVM instance associated to the manager.
        let manager = SimulationManager::default();
        let admin = &manager.agents["admin"];

        // Get a SimulationContract for the Arbiter Math ABI and bytecode.
        let arbiter_math = manager.deployed_contracts.get("arbiter_math").unwrap();

        // Test the cdf function.
        let execution_result = admin.call(arbiter_math, "cdf", I256::from(1).into_tokens())?;
        assert!(execution_result.is_success());
        let unpacked_result = unpack_execution(execution_result)?;
        let output: I256 = arbiter_math.decode_output("cdf", unpacked_result)?;
        println!("cdf(1) = {}", output);
        assert_eq!(output, I256::from(500000000000000000u64));

        // Test the pdf function.
        let execution_result = admin.call(arbiter_math, "pdf", I256::from(1).into_tokens())?;
        assert!(execution_result.is_success());
        let unpacked_result = unpack_execution(execution_result)?;
        let output: I256 = arbiter_math.decode_output("pdf", unpacked_result)?;
        println!("pdf(1) = {}", output);
        assert_eq!(output, I256::from(398942280401432678u64));

        // Test the ppf function.
        let execution_result = admin.call(
            arbiter_math,
            "ppf",
            I256::from(500000000000000000u64).into_tokens(),
        )?;
        let unpacked_result = unpack_execution(execution_result)?;
        let output: I256 = arbiter_math.decode_output("ppf", unpacked_result)?;
        println!("ppf(0.5) = {}", output);

        // Test the mulWadDown function.
        let execution_result = admin.call(
            arbiter_math,
            "mulWadDown",
            (U256::from(1_000_000_000_000_000_000_u128), U256::from(2)).into_tokens(),
        )?;
        let unpacked_result = unpack_execution(execution_result)?;
        let output: U256 = arbiter_math.decode_output("mulWadDown", unpacked_result)?;
        println!("mulWadDown(1, 2) = {}", output);
        assert_eq!(output, U256::from(2));

        // Test the mulWadUp function.
        let execution_result = admin.call(
            arbiter_math,
            "mulWadUp",
            (U256::from(1_000_000_000_000_000_000_u128), U256::from(2)).into_tokens(),
        )?;
        let unpacked_result = unpack_execution(execution_result)?;
        let output: U256 = arbiter_math.decode_output("mulWadUp", unpacked_result)?;
        println!("mulWadUp(1, 2) = {}", output);
        assert_eq!(output, U256::from(2));

        // Test the divWadDown function.
        let execution_result = admin.call(
            arbiter_math,
            "divWadDown",
            (U256::from(1_000_000_000_000_000_000_u128), U256::from(2)).into_tokens(),
        )?;
        let unpacked_result = unpack_execution(execution_result)?;
        let output: U256 = arbiter_math.decode_output("divWadDown", unpacked_result)?;
        println!("divWadDown(1, 2) = {}", output);
        assert_eq!(
            output,
            U256::from(500000000000000000000000000000000000_u128)
        );

        // Test the divWadUp function.
        let execution_result = admin.call(
            arbiter_math,
            "divWadUp",
            (U256::from(1_000_000_000_000_000_000_u128), U256::from(2)).into_tokens(),
        )?;
        let unpacked_result = unpack_execution(execution_result)?;
        let output: U256 = arbiter_math.decode_output("divWadUp", unpacked_result)?;
        println!("divWadUp(1, 2) = {}", output);
        assert_eq!(
            output,
            U256::from(500000000000000000000000000000000000_u128)
        );

        // Test the lnWad function.
        let execution_result = admin.call(
            arbiter_math,
            "log",
            I256::from(1_000_000_000_000_000_000_u128).into_tokens(),
        )?;
        let unpacked_result = unpack_execution(execution_result)?;
        let output = arbiter_math.decode_output::<I256>("log", unpacked_result)?;
        println!("lnWad(0) = {}", output);
        assert_eq!(output, I256::from(0));
        // Test the sqrt function
        let execution_result = admin.call(arbiter_math, "sqrt", U256::from(1u128).into_tokens())?;
        let unpacked_result = unpack_execution(execution_result)?;
        let output = arbiter_math.decode_output::<U256>("sqrt", unpacked_result)?;
        println!("sqrt(1) = {}", output);
        assert_eq!(output, U256::from(1u128));
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use bindings::{arbiter_token, liquid_exchange};
//     use ethers::prelude::U256;
//     use revm::primitives::{ruint::Uint, B160};

//     use crate::{
//         agent::{user::User, Agent, AgentType},
//         environment::contract::SimulationContract,
//         manager::SimulationManager,
//         stochastic::price_process::{PriceProcess, PriceProcessType, GBM},
//         utils::{recast_address, unpack_execution},
//     };

//     #[test]
//     fn swap_x_for_y_liquid_exchange() -> Result<(), Box<dyn Error>> {
//         // define the wad constant
//         let decimals = 18_u8;
//         let wad: U256 = U256::from(10_i64.pow(decimals as u32));

//         // Set up the execution manager and a user address.
//         let mut manager = SimulationManager::default();

//         // Set up a user named alice
//         let user_name = "alice";
//         let user_address = B160::from_low_u64_be(2);
//         let alice = User::new(user_name, None);
//         manager.activate_agent(AgentType::User(alice), user_address)?;
//         let admin = manager.agents.get("admin").unwrap();
//         let alice = manager.agents.get("alice").unwrap();

//         // Create arbiter token general contract.
//         let arbiter_token = SimulationContract::new(
//             arbiter_token::ARBITERTOKEN_ABI.clone(),
//             arbiter_token::ARBITERTOKEN_BYTECODE.clone(),
//         );

//         // Deploy token_x.
//         let name = "Token X";
//         let symbol = "TKNX";
//         let args = (name.to_string(), symbol.to_string(), decimals);
//         let token_x = arbiter_token.deploy(&mut manager.environment, admin, args);

//         // Deploy token_y.
//         let name = "Token Y";
//         let symbol = "TKNY";
//         let args = (name.to_string(), symbol.to_string(), decimals);
//         let token_y = arbiter_token.deploy(&mut manager.environment, admin, args);

//         // Deploy LiquidExchange
//         let price_to_check = 1000;
//         let initial_price = wad.checked_mul(U256::from(price_to_check)).unwrap();
//         let liquid_exchange = SimulationContract::new(
//             liquid_exchange::LIQUIDEXCHANGE_ABI.clone(),
//             bindings::liquid_exchange::LIQUIDEXCHANGE_BYTECODE.clone(),
//         );
//         let args = (
//             recast_address(token_x.address),
//             recast_address(token_y.address),
//             initial_price,
//         );
//         let liquid_exchange_xy = liquid_exchange.deploy(&mut manager.environment, admin, args);

//         // Mint token_x to alice.
//         let mint_amount = wad.checked_mul(U256::from(20)).unwrap(); // in wei units
//         let args = (recast_address(alice.address()), mint_amount);
//         let call_data = token_x.encode_function("mint", args)?;
//         admin.call_contract(&mut manager.environment, &token_x, call_data, Uint::from(0));

//         // Mint max token_y to the liquid_exchange contract.
//         let args = (recast_address(liquid_exchange_xy.address), U256::MAX);
//         let call_data = token_y.encode_function("mint", args)?;
//         admin.call_contract(&mut manager.environment, &token_y, call_data, Uint::from(0));

//         // Have alice's approval for token_x to be spent by the liquid_exchange.
//         let args = (recast_address(liquid_exchange_xy.address), U256::MAX);
//         let call_data = token_x.encode_function("approve", args)?;
//         alice.call_contract(&mut manager.environment, &token_x, call_data, Uint::from(0));

//         // Have alice call the swap function to trade token_x for token_y.
//         let swap_amount = mint_amount / 2;
//         let call_data = liquid_exchange_xy
//             .encode_function("swap", (recast_address(token_x.address), swap_amount))?;
//         alice.call_contract(
//             &mut manager.environment,
//             &liquid_exchange_xy,
//             call_data,
//             Uint::from(0),
//         );

//         // Let alice check they spent the right amount of token_x
//         let call_data = token_x.encode_function("balanceOf", recast_address(user_address))?;
//         let execution_result =
//             alice.call_contract(&mut manager.environment, &token_x, call_data, Uint::from(0)); // Call the 'balanceOf' function.
//         let value = unpack_execution(execution_result)?;
//         let response: U256 = token_x.decode_output("balanceOf", value)?;
//         println!("alice has {} token_x after swap", response);
//         assert_eq!(response, mint_amount - swap_amount);

//         // Let alice check they received the right amount of token_y
//         let call_data = token_y.encode_function("balanceOf", recast_address(user_address))?;
//         let execution_result =
//             alice.call_contract(&mut manager.environment, &token_y, call_data, Uint::from(0)); // Call the 'balanceOf' function.
//         let value = unpack_execution(execution_result)?;
//         let response: U256 = token_y.decode_output("balanceOf", value)?;
//         println!("alice has {} token_y after swap", response);
//         assert_eq!(response, swap_amount * U256::from(price_to_check));
//         Ok(())
//     }

//     #[test]
//     fn swap_y_for_x_liquid_exchange() -> Result<(), Box<dyn Error>> {
//         // define the wad constant
//         let decimals = 18_u8;
//         let wad: U256 = U256::from(10_i64.pow(decimals as u32));

//         // Set up the execution manager and a user address.
//         let mut manager = SimulationManager::default();

//         // Set up a user named alice
//         let user_name = "alice";
//         let user_address = B160::from_low_u64_be(2);
//         let alice = User::new(user_name, None);
//         manager.activate_agent(AgentType::User(alice), user_address)?;
//         let admin = manager.agents.get("admin").unwrap();
//         let alice = manager.agents.get("alice").unwrap();

//         // Create arbiter token general contract.
//         let arbiter_token = SimulationContract::new(
//             arbiter_token::ARBITERTOKEN_ABI.clone(),
//             arbiter_token::ARBITERTOKEN_BYTECODE.clone(),
//         );

//         // Deploy token_x.
//         let name = "Token X";
//         let symbol = "TKNX";
//         let args = (name.to_string(), symbol.to_string(), decimals);
//         let token_x = arbiter_token.deploy(&mut manager.environment, admin, args);

//         // Deploy token_y.
//         let name = "Token Y";
//         let symbol = "TKNY";
//         let args = (name.to_string(), symbol.to_string(), decimals);
//         let token_y = arbiter_token.deploy(&mut manager.environment, admin, args);

//         // Deploy LiquidExchange
//         let price_to_check = 1000;
//         let initial_price = wad.checked_mul(U256::from(price_to_check)).unwrap();
//         let liquid_exchange = SimulationContract::new(
//             liquid_exchange::LIQUIDEXCHANGE_ABI.clone(),
//             liquid_exchange::LIQUIDEXCHANGE_BYTECODE.clone(),
//         );
//         let args = (
//             recast_address(token_x.address),
//             recast_address(token_y.address),
//             initial_price,
//         );
//         let liquid_exchange_xy = liquid_exchange.deploy(&mut manager.environment, admin, args);

//         // Mint token_y to alice.
//         let mint_amount = wad.checked_mul(U256::from(20)).unwrap(); // in wei units
//         let args = (recast_address(alice.address()), mint_amount);
//         let call_data = token_y.encode_function("mint", args)?;
//         admin.call_contract(&mut manager.environment, &token_y, call_data, Uint::from(0));

//         // Mint max token_x to the liquid_exchange contract.
//         let args = (recast_address(liquid_exchange_xy.address), U256::MAX);
//         let call_data = token_x.encode_function("mint", args)?;
//         admin.call_contract(&mut manager.environment, &token_x, call_data, Uint::from(0));

//         // Have alice's approval for token_y to be spent by the liquid_exchange.
//         let args = (recast_address(liquid_exchange_xy.address), U256::MAX);
//         let call_data = token_y.encode_function("approve", args)?;
//         alice.call_contract(&mut manager.environment, &token_y, call_data, Uint::from(0));

//         // Have alice call the swap function to trade token_y for token_x.
//         let swap_amount = mint_amount / 2;
//         let call_data = liquid_exchange_xy
//             .encode_function("swap", (recast_address(token_y.address), swap_amount))?;
//         alice.call_contract(
//             &mut manager.environment,
//             &liquid_exchange_xy,
//             call_data,
//             Uint::from(0),
//         );

//         // Let alice check they spent the right amount of token_y
//         let call_data = token_y.encode_function("balanceOf", recast_address(user_address))?;
//         let execution_result =
//             alice.call_contract(&mut manager.environment, &token_y, call_data, Uint::from(0)); // Call the 'balanceOf' function.
//         let value = unpack_execution(execution_result)?;
//         let response: U256 = token_y.decode_output("balanceOf", value)?;
//         println!("alice has {} token_y after swap", response);
//         assert_eq!(response, mint_amount - swap_amount);

//         // Let alice check they received the right amount of token_x
//         let call_data = token_x.encode_function("balanceOf", recast_address(user_address))?;
//         let execution_result =
//             alice.call_contract(&mut manager.environment, &token_x, call_data, Uint::from(0)); // Call the 'balanceOf' function.
//         let value = unpack_execution(execution_result)?;
//         let response: U256 = token_x.decode_output("balanceOf", value)?;
//         println!("alice has {} token_x after swap", response);
//         assert_eq!(response, U256::from(10_i64.pow(16)));
//         Ok(())
//     }

//     #[test]
//     fn price_simulation_oracle() -> Result<(), Box<dyn Error>> {
//         // Get a price path from the oracle.
//         let timestep = 1.0;
//         let timescale = String::from("day");
//         let num_steps = 7;
//         let initial_price = 100.0;
//         let drift = 0.5;
//         let volatility = 0.1;
//         let seed = 123;
//         let price = PriceProcess::new(
//             PriceProcessType::GBM(GBM::new(drift, volatility)),
//             timestep,
//             timescale,
//             num_steps,
//             initial_price,
//             seed,
//         );
//         let price_path = price.generate_price_path();

//         // Set up the liquid exchange
//         // define the wad constant
//         let decimals = 18_u8;
//         let wad: U256 = U256::from(10_i64.pow(decimals as u32));

//         // Set up the execution manager and a user address.
//         let mut manager = SimulationManager::default();
//         let admin = manager.agents.get("admin").unwrap();

//         // Create arbiter token general contract.
//         let arbiter_token = SimulationContract::new(
//             arbiter_token::ARBITERTOKEN_ABI.clone(),
//             bindings::arbiter_token::ARBITERTOKEN_BYTECODE.clone(),
//         );

//         // Deploy token_x.
//         let name = "Token X";
//         let symbol = "TKNX";
//         let args = (name.to_string(), symbol.to_string(), decimals);
//         let token_x = arbiter_token.deploy(&mut manager.environment, admin, args);

//         // Deploy token_y.
//         let name = "Token Y";
//         let symbol = "TKNY";
//         let args = (name.to_string(), symbol.to_string(), decimals);
//         let token_y = arbiter_token.deploy(&mut manager.environment, admin, args);

//         // Deploy LiquidExchange
//         let price_to_check = 1000; // Initial price is this for sake of ease.
//         let initial_price = wad.checked_mul(U256::from(price_to_check)).unwrap();
//         let liquid_exchange = SimulationContract::new(
//             liquid_exchange::LIQUIDEXCHANGE_ABI.clone(),
//             bindings::liquid_exchange::LIQUIDEXCHANGE_BYTECODE.clone(),
//         );
//         let args = (
//             recast_address(token_x.address),
//             recast_address(token_y.address),
//             initial_price,
//         );
//         let liquid_exchange_xy = liquid_exchange.deploy(&mut manager.environment, admin, args);

//         // Loop over and set prices on the liquid exchange from the oracle.
//         for price in price_path.1 {
//             println!("Price from price path: {}", price);
//             let wad_price = crate::utils::float_to_wad(price);
//             println!("WAD price: {}", wad_price);
//             let call_data = liquid_exchange_xy.encode_function("setPrice", wad_price)?;
//             admin.call_contract(
//                 &mut manager.environment,
//                 &liquid_exchange_xy,
//                 call_data,
//                 Uint::from(0),
//             );
//             // Check that the price is set correctly
//             let call_data = liquid_exchange_xy.encode_function("price", ())?;
//             let execution_result = admin.call_contract(
//                 &mut manager.environment,
//                 &liquid_exchange_xy,
//                 call_data,
//                 Uint::from(0),
//             );
//             let value = unpack_execution(execution_result)?;
//             let response: U256 = liquid_exchange_xy.decode_output("price", value)?;
//             println!("Price from the exchange: {}", response);
//             assert_eq!(response, wad_price);
//         }
//         Ok(())
//     }
// }
