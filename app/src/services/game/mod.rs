use crate::admin::Admins;
use crate::services::{gold_vft::GoldService, item_vmt::ItemService};
use gstd::{exec, msg};
use sails_rs::hex;
use sails_rs::{collections::HashMap, gstd::service, prelude::*};
use schnorrkel::{PublicKey, Signature};
use vmt_service::utils::TokenId;

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub enum GameStatus {
    Created,
    InProgress,
    Ended,
}

impl Default for GameStatus {
    fn default() -> Self {
        GameStatus::Created
    }
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Default)]
pub struct GameInfo {
    stage: u32,
    time: u32,
    status: GameStatus,
    score: i32,
    creator: ActorId,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Default)]
pub struct GameSettings {
    verifier_public_key: Option<Vec<u8>>,
    game_time: u32,
    max_earn: u32,
    initial_max_stamina: u64,
    stamina_recovery_rate: u64,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, Default)]
pub struct Player {
    name: String,
    avatar_id: u32,
    avatar_icon: String,
    highest_score: i32,
    games_played: u32,
    stamina: u64,
    max_stamina: u64,
    min_stamina_block: u64,
}

#[derive(Default)]
pub struct GameStorage {
    games: HashMap<u32, GameInfo>,
    settings: GameSettings,
    players: HashMap<ActorId, Player>,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub enum GameEvent {
    GameCreated {
        game_id: u32,
        creator: ActorId,
    },
    GameUpdated {
        game_id: u32,
        score: i32,
        earn: U256,
    },
}

static mut GAME_STORAGE: Option<GameStorage> = None;

#[derive(Clone)]
pub struct GameService {}

impl GameService {
    pub fn seed() -> Self {
        unsafe {
        
            GAME_STORAGE = Some(GameStorage {
                games: HashMap::new(),
                settings:     GameSettings {
                    verifier_public_key: Some(hex::decode("b8c4cd5e14f7ae7cab1b9d1ce101648f96295a4805abc95bdf907740f8985220").expect("Decoding failed")),
                    game_time: 60,
                    max_earn: 2000,
                    initial_max_stamina: 5,
                    stamina_recovery_rate: 1800000,
                },
                players: HashMap::new(),
            });
        };
        GameService {}
    }

    pub fn get_mut(&mut self) -> &'static mut GameStorage {
        unsafe {
            GAME_STORAGE
                .as_mut()
                .expect("GameInfo storage is not initialized")
        }
    }

    pub fn get(&self) -> &'static GameStorage {
        unsafe {
            GAME_STORAGE
                .as_ref()
                .expect("GameInfo storage is not initialized")
        }
    }
}

#[service(events = GameEvent)]
impl GameService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_game(&self, game_id: u32) -> Option<GameInfo> {
        let storage = self.get();
        storage.games.get(&game_id).cloned()
    }

    pub fn set_verifier_public_key(&mut self, public_key: Vec<u8>) {
        self.ensure_is_admin();
        let storage = self.get_mut();
        storage.settings.verifier_public_key = Some(public_key);
    }

    pub fn set_game_time(&mut self, game_time: u32) {
        self.ensure_is_admin();
        let storage = self.get_mut();
        storage.settings.game_time = game_time;
    }

    pub fn set_max_earn(&mut self, max_earn: u32) {
        self.ensure_is_admin();
        let storage = self.get_mut();
        storage.settings.max_earn = max_earn;
    }

    pub fn set_initial_max_stamina(&mut self, initial_max_stamina: u64) {
        self.ensure_is_admin();
        let storage = self.get_mut();
        storage.settings.initial_max_stamina = initial_max_stamina;
    }

    pub fn set_stamina_recovery_rate(&mut self, stamina_recovery_rate: u64) {
        self.ensure_is_admin();
        let storage = self.get_mut();
        storage.settings.stamina_recovery_rate = stamina_recovery_rate;
    }

    pub fn get_settings(&self) -> GameSettings {
        let storage = self.get();
        storage.settings.clone()
    }

    pub fn create_game(&mut self) -> u32 {
        let storage = self.get_mut();
        let player_id = msg::source();

        // Check if the player exists
        let player = storage
            .players
            .get_mut(&player_id)
            .expect("Player not registered");

        // Calculate the current block height
        let current_block = exec::block_timestamp();

        if player.min_stamina_block == 0 {
            player.min_stamina_block = current_block;
        } else {
            // Calculate the recovered stamina
            let (recovered_stamina, remaining_blocks, _) = self.calculate_stamina(player);

            // Update player's stamina
            player.stamina = (player.stamina + recovered_stamina).min(player.max_stamina);

            // Update min_stamina_block if any stamina was recovered
            if player.stamina >= player.max_stamina {
                player.min_stamina_block = current_block;
            } else {
                player.min_stamina_block = current_block - remaining_blocks;
            }
    
        }

        // Check if the player has enough stamina
        if player.stamina == 0 {
            panic!("Not enough stamina to start a game");
        }
        // Deduct 1 stamina
        player.stamina -= 1;

        let game_id = storage.games.len() as u32 + 1;
        let game = GameInfo {
            stage: 0,
            time: storage.settings.game_time,
            status: GameStatus::Created,
            score: 0,
            creator: player_id,
        };
        storage.games.insert(game_id, game);

        // Emit GameCreated event
        self.notify_on(GameEvent::GameCreated { game_id, creator: player_id })
        .expect("Notification Error");


        game_id
    }

    pub fn update_game(
        &mut self,
        game_id: u32,
        score: i32,
        earn: U256,
        sign: Vec<u8>,
        token_ids: Vec<TokenId>,
        amounts: Vec<U256>,
    ) {
        let storage = self.get_mut();
        if let Some(game) = storage.games.get_mut(&game_id) {
            // if earn > storage.settings.max_earn.into() {
            //     panic!("Earn exceeds max earn");
            // }
            let new_earn = earn.min(storage.settings.max_earn.into());

            // Verify the sign
            let message = format!("{}{}{}", game_id, score, earn);
            let public_key_bytes = storage
                .settings
                .verifier_public_key
                .as_ref()
                .expect("Verifier public key not set");
            let public_key = PublicKey::from_bytes(public_key_bytes).expect("Invalid public key");
            let signature = Signature::from_bytes(&sign).expect("Invalid signature");
            if public_key
                .verify_simple(b"substrate", message.as_bytes(), &signature)
                .is_err()
            {
                panic!("Invalid signature");
            }

            // Ensure token_ids and amounts have the same length
            if token_ids.len() != amounts.len() {
                panic!("Token IDs and amounts length mismatch");
            }

            // Burn items
            for (token_id, amount) in token_ids.iter().zip(amounts.iter()) {
                ItemService::burn_internal_notify_off(
                    ItemService::get_item(),
                    game.creator,
                    *token_id,
                    *amount,
                );
            }

            // Earn gold
            GoldService::mint_internal_notify_off(game.creator, new_earn);

            game.score = score;
            game.status = GameStatus::Ended;

            // Update player's highest score and games played
            let player = storage
                .players
                .get_mut(&game.creator)
                .expect("Player not found");
            player.games_played += 1;
            if score > player.highest_score {
                player.highest_score = score;
            }

            // Emit GameUpdated event
            self.notify_on(GameEvent::GameUpdated { game_id, score, earn: new_earn })
            .expect("Notification Error");
        }
    }

    pub fn register_player(&mut self, name: String, avatar_id: u32, avatar_icon: String) {
        let storage = self.get_mut();
        let player = Player {
            name,
            avatar_id,
            avatar_icon,
            highest_score: 0,
            games_played: 0,
            stamina: storage.settings.initial_max_stamina,
            max_stamina: storage.settings.initial_max_stamina,
            min_stamina_block: 0,
        };
        storage.players.insert(msg::source(), player);
    }

    pub fn update_player_info(
        &mut self,
        name: Option<String>,
        avatar_id: Option<u32>,
        avatar_icon: Option<String>,
    ) {
        let storage = self.get_mut();
        if let Some(player) = storage.players.get_mut(&msg::source()) {
            if let Some(name) = name {
                player.name = name;
            }
            if let Some(avatar_id) = avatar_id {
                player.avatar_id = avatar_id;
            }
            if let Some(avatar_icon) = avatar_icon {
                player.avatar_icon = avatar_icon;
            }
        }
    }

    pub fn get_player(&self, player_id: ActorId) -> Option<Player> {
        let storage = self.get();
        storage.players.get(&player_id).cloned()
    }

    pub fn get_players(&self) -> Vec<(ActorId, Player)> {
        let storage = self.get();
        storage
            .players
            .iter()
            .map(|(id, player)| (*id, player.clone()))
            .collect()
    }

    pub fn get_leaderboard(&self) -> Vec<(ActorId, i32)> {
        let storage = self.get();
        let mut leaderboard: Vec<(ActorId, i32)> = storage
            .players
            .iter()
            .map(|(id, player)| (*id, player.highest_score))
            .collect();
        leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
        leaderboard
    }

    pub fn get_player_stamina(&self) -> u64 {
        let storage = self.get();
        let player = storage
            .players
            .get(&msg::source())
            .expect("Player not registered");

        if player.min_stamina_block == 0 {
            player.stamina
        } else {
            let (recovered_stamina, _, _) = self.calculate_stamina(player);
            let new_stamina = (player.stamina + recovered_stamina).min(player.max_stamina);
            new_stamina
        }
    }

    pub fn get_player_recovered_block(&self) -> u64 {
        let storage = self.get();
        let player = storage
            .players
            .get(&msg::source())
            .expect("Player not registered");

        if player.min_stamina_block == 0 {
            0
        } else {
            let (recovered_stamina, remaining_blocks, _) = self.calculate_stamina(player);
            let new_stamina = (player.stamina + recovered_stamina).min(player.max_stamina);
            if new_stamina >= player.max_stamina {
                0
            } else {
                storage.settings.stamina_recovery_rate - remaining_blocks
            }
        }
    }
}

impl GameService {
    fn ensure_is_admin(&self) {
        if !Admins::is_admin(&msg::source()) {
            panic!("Not admin")
        };
    }

    fn calculate_stamina(&self, player: &Player) -> (u64, u64, u64) {
        let current_block = exec::block_timestamp();
        let blocks_passed = current_block - player.min_stamina_block;
        let recovered_stamina = blocks_passed / self.get().settings.stamina_recovery_rate;
        let remaining_blocks = blocks_passed % self.get().settings.stamina_recovery_rate;
        (recovered_stamina, remaining_blocks, current_block)
    }
}