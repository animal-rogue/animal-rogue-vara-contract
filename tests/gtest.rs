use animal_rogue_client::{ traits::*, GameStatus, TokenMetadata};
use rand_core::OsRng;
use sails_rs::{
    calls::*, gtest::{calls::*, System}, hex, ActorId, U256
};
use schnorrkel::Keypair;
use schnorrkel::{PublicKey, Signature};

const ACTOR_ID: u64 = 42;
const NEW_ADMIN_ID: u64 = 43;
const RECIPIENT_ID: u64 = 44;

const TOKEN_ID: u64 = 1;

#[tokio::test]
async fn add_admin_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Admin::new(remoting.clone());
    // Add new admin
    let result = service_client
        .add_admin(NEW_ADMIN_ID.into())
        .send_recv(program_id)
        .await
        .unwrap();

    assert!(result);

    // Verify the new admin is added
    let is_admin = service_client
        .is_admin(NEW_ADMIN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert!(is_admin);
}

#[tokio::test]
async fn remove_admin_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Admin::new(remoting.clone());

    // Add new admin first
    service_client
        .add_admin(NEW_ADMIN_ID.into())
        .send_recv(program_id)
        .await
        .unwrap();

    // Remove the new admin
    let result = service_client
        .remove_admin(NEW_ADMIN_ID.into())
        .send_recv(program_id)
        .await
        .unwrap();

    assert!(result);

    // Verify the admin is removed
    let is_admin = service_client
        .is_admin(NEW_ADMIN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert!(!is_admin);
}

#[tokio::test]
async fn is_admin_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let service_client = animal_rogue_client::Admin::new(remoting.clone());

    // Verify the deployer is admin
    let is_admin = service_client
        .is_admin(ACTOR_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert!(is_admin);
}

//   *******************************      gold_vft      *******************************
#[tokio::test]
async fn mint_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Vft::new(remoting.clone());

    // Mint tokens
    let result = service_client
        .mint(RECIPIENT_ID.into(), 1000.into())
        .send_recv(program_id)
        .await
        .unwrap();

    assert!(result);

    // Verify the balance
    let balance = service_client
        .balance_of(RECIPIENT_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 1000.into());
}

#[tokio::test]
async fn burn_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Vft::new(remoting.clone());

    // Mint tokens first
    service_client
        .mint(RECIPIENT_ID.into(), 1000.into())
        .send_recv(program_id)
        .await
        .unwrap();

    // Burn tokens
    let result = service_client
        .burn(RECIPIENT_ID.into(), 500.into())
        .send_recv(program_id)
        .await
        .unwrap();

    assert!(result);

    // Verify the balance
    let balance = service_client
        .balance_of(RECIPIENT_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 500.into());
}

#[tokio::test]
async fn transfer_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Vft::new(remoting.clone());

    // Mint tokens first
    service_client
        .mint(ACTOR_ID.into(), 1000.into())
        .send_recv(program_id)
        .await
        .unwrap();

    // Transfer tokens
    let result = service_client
        .transfer(RECIPIENT_ID.into(), 500.into())
        .send_recv(program_id)
        .await
        .unwrap();

    assert!(result);

    // Verify the balances
    let balance_actor = service_client
        .balance_of(ACTOR_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    let balance_recipient = service_client
        .balance_of(RECIPIENT_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance_actor, 500.into());
    assert_eq!(balance_recipient, 500.into());
}

//   *******************************      item_vmt      *******************************

#[tokio::test]
async fn create_token_metadata_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Vmt::new(remoting.clone());

    // Create token metadata
    let metadata = TokenMetadata {
        title: Some("Sample title field".to_string()),
        description: Some("Sample description".to_string()),
        media: Some("Sample media".to_string()),
        reference: Some("Sample reference".to_string()),
        // Replace with actual field name and value
    };

    service_client
        .create_token_metadata(TOKEN_ID.into(), metadata)
        .send_recv(program_id)
        .await
        .unwrap();

    // Mint tokens
    let result = service_client
        .mint(RECIPIENT_ID.into(), TOKEN_ID.into(), 1000.into()) // 假设 mint 方法存在
        .send_recv(program_id)
        .await;

    assert!(result.is_ok());

    // Verify the balance
    let balance = service_client
        .balance_of(RECIPIENT_ID.into(), TOKEN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 1000.into());
}

#[tokio::test]
async fn vmt_mint_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Vmt::new(remoting.clone());

    // Create token metadata
    let metadata = TokenMetadata {
        title: Some("Sample title field".to_string()),
        description: Some("Sample description".to_string()),
        media: Some("Sample media".to_string()),
        reference: Some("Sample reference".to_string()),
        // Replace with actual field name and value
    };
    service_client
        .create_token_metadata(TOKEN_ID.into(), metadata)
        .send_recv(program_id)
        .await
        .unwrap();

    // Mint tokens
    let result = service_client
        .mint(RECIPIENT_ID.into(), TOKEN_ID.into(), 1000.into()) // 假设 mint 方法存在
        .send_recv(program_id)
        .await;

    assert!(result.is_ok());

    // Verify the balance
    let balance = service_client
        .balance_of(RECIPIENT_ID.into(), TOKEN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 1000.into());
}

async fn vmt_mint_batch(
    remoting: GTestRemoting,
    recipt_id: u64,
    program_id: ActorId,
) -> animal_rogue_client::Vmt<GTestRemoting> {
    let mut service_client = animal_rogue_client::Vmt::new(remoting.clone());

    // Create token metadata
    let metadata_1 = TokenMetadata {
        title: Some("Sample title field 1".to_string()),
        description: Some("Sample description 1".to_string()),
        media: Some("Sample media 1".to_string()),
        reference: Some("Sample reference 1".to_string()),
    };
    let metadata_2 = TokenMetadata {
        title: Some("Sample title field 2".to_string()),
        description: Some("Sample description 2".to_string()),
        media: Some("Sample media 2".to_string()),
        reference: Some("Sample reference 2".to_string()),
    };
    service_client
        .create_token_metadata(TOKEN_ID.into(), metadata_1)
        .send_recv(program_id)
        .await
        .unwrap();
    service_client
        .create_token_metadata((TOKEN_ID + 1).into(), metadata_2)
        .send_recv(program_id)
        .await
        .unwrap();

    // Mint batch tokens
    let result = service_client
        .mint_batch(
            recipt_id.into(),
            vec![TOKEN_ID.into(), (TOKEN_ID + 1).into()],
            vec![1000.into(), 2000.into()],
        ) // 假设 mint_batch 方法存在
        .send_recv(program_id)
        .await;

    assert!(result.is_ok());

    // Verify the balances
    let balance_1 = service_client
        .balance_of(recipt_id.into(), TOKEN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    let balance_2 = service_client
        .balance_of(recipt_id.into(), (TOKEN_ID + 1).into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance_1, 1000.into());
    assert_eq!(balance_2, 2000.into());

    service_client
}

#[tokio::test]
async fn vmt_mint_batch_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    vmt_mint_batch(remoting, RECIPIENT_ID, program_id).await;
}

#[tokio::test]
async fn vmt_burn_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Vmt::new(remoting.clone());

    // Create token metadata
    let metadata = TokenMetadata {
        title: Some("Sample title field".to_string()),
        description: Some("Sample description".to_string()),
        media: Some("Sample media".to_string()),
        reference: Some("Sample reference".to_string()),
        // Replace with actual field name and value
    };
    service_client
        .create_token_metadata(TOKEN_ID.into(), metadata)
        .send_recv(program_id)
        .await
        .unwrap();

    // Mint tokens first
    service_client
        .mint(RECIPIENT_ID.into(), TOKEN_ID.into(), 1000.into()) // 假设 mint 方法存在
        .send_recv(program_id)
        .await
        .unwrap();

    // Burn tokens
    let result = service_client
        .burn(RECIPIENT_ID.into(), TOKEN_ID.into(), 500.into()) // 假设 burn 方法存在
        .send_recv(program_id)
        .await;

    assert!(result.is_ok(), "Burn failed: {:?}", result);
    // Verify the balance
    let balance = service_client
        .balance_of(RECIPIENT_ID.into(), TOKEN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 500.into());
}

#[tokio::test]
async fn vmt_burn_batch_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Vmt::new(remoting.clone());

    // Create token metadata
    let metadata_1 = TokenMetadata {
        title: Some("Sample title field 1".to_string()),
        description: Some("Sample description 1".to_string()),
        media: Some("Sample media 1".to_string()),
        reference: Some("Sample reference 1".to_string()),
    };
    let metadata_2 = TokenMetadata {
        title: Some("Sample title field 2".to_string()),
        description: Some("Sample description 2".to_string()),
        media: Some("Sample media 2".to_string()),
        reference: Some("Sample reference 2".to_string()),
    };
    service_client
        .create_token_metadata(TOKEN_ID.into(), metadata_1)
        .send_recv(program_id)
        .await
        .unwrap();
    service_client
        .create_token_metadata((TOKEN_ID + 1).into(), metadata_2)
        .send_recv(program_id)
        .await
        .unwrap();

    // Mint batch tokens first
    service_client
        .mint_batch(
            RECIPIENT_ID.into(),
            vec![TOKEN_ID.into(), (TOKEN_ID + 1).into()],
            vec![1000.into(), 2000.into()],
        )
        .send_recv(program_id)
        .await
        .unwrap();

    // Burn batch tokens
    let result = service_client
        .burn_batch(
            RECIPIENT_ID.into(),
            vec![TOKEN_ID.into(), (TOKEN_ID + 1).into()],
            vec![500.into(), 1000.into()],
        )
        .send_recv(program_id)
        .await;

    assert!(result.is_ok(), "Burn batch failed: {:?}", result);

    // Verify the balances
    let balance_1 = service_client
        .balance_of(RECIPIENT_ID.into(), TOKEN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    let balance_2 = service_client
        .balance_of(RECIPIENT_ID.into(), (TOKEN_ID + 1).into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance_1, 500.into());
    assert_eq!(balance_2, 1000.into());
}

//   *******************************      market      *******************************

#[tokio::test]
async fn market_set_price_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = animal_rogue_client::Market::new(remoting.clone());

    // Set price for a token
    let price: U256 = 100.into();
    let result = service_client
        .set_price(TOKEN_ID.into(), price)
        .send_recv(program_id)
        .await;

    assert!(result.is_ok(), "set_price failed: {:?}", result);

    // Verify the price is set correctly
    let stored_price = service_client
        .get_price(TOKEN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(stored_price.unwrap(), price);
}

#[tokio::test]
async fn market_buy_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut market_client = animal_rogue_client::Market::new(remoting.clone());
    let mut gold_client = animal_rogue_client::Vft::new(remoting.clone());
    let item_client = animal_rogue_client::Vmt::new(remoting.clone());

    // Set price for a token
    let price: U256 = 100.into();
    market_client
        .set_price(TOKEN_ID.into(), price)
        .send_recv(program_id)
        .await
        .unwrap();

    // Mint some gold tokens to the buyer
    let gold_amount: U256 = 1000.into();
    gold_client
        .mint(ACTOR_ID.into(), gold_amount)
        .send_recv(program_id)
        .await
        .unwrap();

    // Perform a buy operation
    let amount_to_buy: U256 = 5.into();
    let result = market_client
        .buy(TOKEN_ID.into(), amount_to_buy)
        .send_recv(program_id)
        .await;

    assert!(result.is_ok(), "buy failed: {:?}", result);

    // Verify the buyer's gold balance after purchase
    let buyer_balance_after = gold_client
        .balance_of(ACTOR_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    let total_cost = price * amount_to_buy;
    assert_eq!(buyer_balance_after, gold_amount - total_cost);

    // Verify the buyer's item balance
    let item_balance = item_client
        .balance_of(ACTOR_ID.into(), TOKEN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(item_balance, amount_to_buy);
}

//   *******************************      game      *******************************

#[tokio::test]
async fn create_game_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut game_client = animal_rogue_client::Game::new(remoting.clone());

    let stamina = 10;
    let single_block_time = 3000;
    let set_stamina_recovery_rate = single_block_time * 10;
    let mut current_block = 0;

    // Set game settings
    game_client
        .set_game_time(60)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_max_earn(1000)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_initial_max_stamina(stamina)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_stamina_recovery_rate(set_stamina_recovery_rate)
        .send_recv(program_id)
        .await
        .unwrap();

    // Register a player
    game_client
        .register_player("Player1".to_string(), 1, "avatar1".to_string())
        .send_recv(program_id)
        .await
        .unwrap();

    // Create a game
    let result = game_client.create_game().send_recv(program_id).await;
    assert!(result.is_ok(), "create_game failed: {:?}", result);
    current_block += 1;

    let game_id = result.unwrap();
    // Verify the game is created
    let game = game_client
        .get_game(game_id)
        .recv(program_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(game.stage, 0);
    assert_eq!(
        game.time,
        game_client
            .get_settings()
            .recv(program_id)
            .await
            .unwrap()
            .game_time
    );
    current_block += 2;
    assert_eq!(game.status, GameStatus::Created);
    assert_eq!(game.score, 0);
    assert_eq!(game.creator, ACTOR_ID.into());

    // Check player recovered_block after block advancement
    let recovered_block = game_client
        .get_player_recovered_block()
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(
        recovered_block,
        set_stamina_recovery_rate - current_block * single_block_time
    );

    current_block += 1;

    // Check player stamina after block advancement
    let stamina_after_blocks = game_client
        .get_player_stamina()
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(stamina_after_blocks, stamina - 1);

    current_block += 1;

    // after 5 blocks, stamina should be recovered
    remoting
        .system()
        .run_to_block(remoting.system().block_height() + 5 as u32);

    current_block += 5;
    let recovered_block = game_client
        .get_player_recovered_block()
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(
        recovered_block,
        set_stamina_recovery_rate - current_block * single_block_time
    );

    let stamina_after_blocks = game_client
        .get_player_stamina()
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(stamina_after_blocks, stamina);

    // Create another game
    let result = game_client.create_game().send_recv(program_id).await;

    assert!(result.is_ok(), "create_game failed: {:?}", result);
    let game_id = result.unwrap();

    // Verify the game is created
    let game = game_client
        .get_game(game_id)
        .recv(program_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(game.stage, 0);
    assert_eq!(
        game.time,
        game_client
            .get_settings()
            .recv(program_id)
            .await
            .unwrap()
            .game_time
    );
    assert_eq!(game.status, GameStatus::Created);
    assert_eq!(game.score, 0);
    assert_eq!(game.creator, ACTOR_ID.into());

    // Check player stamina after creating another game
    let stamina_after_blocks = game_client
        .get_player_stamina()
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(stamina_after_blocks, stamina - 1);

    // Create another game
    let result = game_client.create_game().send_recv(program_id).await;

    assert!(result.is_ok(), "create_game failed: {:?}", result);

    // Check player stamina after creating another game
    let stamina_after_blocks = game_client
        .get_player_stamina()
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(stamina_after_blocks, stamina - 2);
}

#[tokio::test]
async fn update_game_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut game_client = animal_rogue_client::Game::new(remoting.clone());

    // Mint the item
    let item_service_client = vmt_mint_batch(remoting.clone(), ACTOR_ID, program_id).await;

    // Set game settings
    game_client
        .set_game_time(60)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_max_earn(1000)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_initial_max_stamina(100)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_stamina_recovery_rate(1000000)
        .send_recv(program_id)
        .await
        .unwrap();

    // Register a player
    game_client
        .register_player("Player1".to_string(), 1, "avatar1".to_string())
        .send_recv(program_id)
        .await
        .unwrap();

    // Generate keypair
    let keypair: Keypair = Keypair::generate_with(OsRng);

    // Set verifier public key
    let public_key = keypair.public.to_bytes().to_vec();
    game_client
        .set_verifier_public_key(public_key.clone())
        .send_recv(program_id)
        .await
        .unwrap();

    // Create a game
    let result = game_client.create_game().send_recv(program_id).await;

    assert!(result.is_ok(), "create_game failed: {:?}", result);
    let game_id = result.unwrap();

    // Prepare signature
    let message_str = format!("{}{}{}", game_id, 100, U256::from(50));
    let message = message_str.as_bytes();
    let signature = keypair.sign_simple(b"substrate", &message).to_bytes().to_vec();

    // Update the game
    let token_ids = vec![TOKEN_ID.into()];
    let amounts = vec![U256::from(10)];
    let result = game_client
        .update_game(game_id, 100, U256::from(50), signature, token_ids, amounts)
        .send_recv(program_id)
        .await;
    assert!(result.is_ok(), "update_game failed: {:?}", result);

    // Verify the game is updated
    let game = game_client
        .get_game(game_id)
        .recv(program_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(game.score, 100);
    assert_eq!(game.status, GameStatus::Ended);

    // Verify the player's highest score and games played
    let player = game_client
        .get_player(ACTOR_ID.into())
        .recv(program_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(player.highest_score, 100);
    assert_eq!(player.games_played, 1);

    // Verify the vmt item's balance
    let balance_1 = item_service_client
        .balance_of(ACTOR_ID.into(), TOKEN_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance_1, 990.into());

    // Verify the gold token balance
    let token_service_client = animal_rogue_client::Vft::new(remoting.clone());
    // Verify the balance
    let balance = token_service_client
        .balance_of(ACTOR_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(balance, 50.into());

    // Verify the leaderboard
    let leaderboard = game_client
        .get_leaderboard()
        .recv(program_id)
        .await
        .unwrap();
    assert_eq!(leaderboard.len(), 1);
    assert_eq!(leaderboard[0].1, 100);

}

#[tokio::test]
async fn register_player_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut game_client = animal_rogue_client::Game::new(remoting.clone());

    // Set game settings
    game_client
        .set_game_time(60)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_max_earn(1000)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_initial_max_stamina(100)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_stamina_recovery_rate(10)
        .send_recv(program_id)
        .await
        .unwrap();

    // Register a player
    game_client
        .register_player("Player1".to_string(), 1, "avatar1".to_string())
        .send_recv(program_id)
        .await
        .unwrap();

    // Verify the player is registered
    let player = game_client
        .get_player(ACTOR_ID.into())
        .recv(program_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(player.name, "Player1");
    assert_eq!(player.avatar_id, 1);
    assert_eq!(player.avatar_icon, "avatar1");
    assert_eq!(player.highest_score, 0);
    assert_eq!(player.games_played, 0);
    assert_eq!(
        player.stamina,
        game_client
            .get_settings()
            .recv(program_id)
            .await
            .unwrap()
            .initial_max_stamina
    );
    assert_eq!(
        player.max_stamina,
        game_client
            .get_settings()
            .recv(program_id)
            .await
            .unwrap()
            .initial_max_stamina
    );
}

#[tokio::test]
async fn update_player_info_works() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let remoting = GTestRemoting::new(system, ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(animal_rogue::WASM_BINARY);

    let program_factory = animal_rogue_client::AnimalRogueFactory::new(remoting.clone());

    let program_id = program_factory
        .new() // Call program's constructor
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut game_client = animal_rogue_client::Game::new(remoting.clone());

    // Set game settings
    game_client
        .set_game_time(60)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_max_earn(1000)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_initial_max_stamina(100)
        .send_recv(program_id)
        .await
        .unwrap();
    game_client
        .set_stamina_recovery_rate(10)
        .send_recv(program_id)
        .await
        .unwrap();

    // Register a player
    game_client
        .register_player("Player1".to_string(), 1, "avatar1".to_string())
        .send_recv(program_id)
        .await
        .unwrap();

    // Update player info
    game_client
        .update_player_info(
            Some("Player2".to_string()),
            Some(2),
            Some("avatar2".to_string()),
        )
        .send_recv(program_id)
        .await
        .unwrap();

    // Verify the player info is updated
    let player = game_client
        .get_player(ACTOR_ID.into())
        .recv(program_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(player.name, "Player2");
    assert_eq!(player.avatar_id, 2);
    assert_eq!(player.avatar_icon, "avatar2");
}

#[tokio::test]
async fn verifier_public_key_test() {
    let pk_hex = "ec4903b024f78c7fedebaa872254745f7485c96de9e883ded378d5745f6af220";
    let sig_hex = "76744dd0c5a0c365bfa2ae923738f0d0dc93ce0ad5ce876cc67052e5923ddf298d021182b2cb037f160da1b693688c4d1c6865839a27d97ff33080ce27315886";
    let msg = "Hello, world!";

    let public_key_bytes = hex::decode(pk_hex).expect("Invalid public key hex");
    let signature_bytes = hex::decode(sig_hex).expect("Invalid signature hex");

    let public_key = PublicKey::from_bytes(&public_key_bytes).expect("Invalid public key");
    let signature = Signature::from_bytes(&signature_bytes).expect("Invalid signature");

    if public_key
        .verify_simple(b"substrate", msg.as_bytes(), &signature)
        .is_err()
    {
        panic!("Invalid signature");
    } else {
        println!("Signature is valid");
    }
}