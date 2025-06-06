#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Player {
    pub address: Address,
    pub username: String,
    pub level: u32,
    pub experience: u64,
    pub total_mined: u64,
    pub active_mines: Vec<u32>,
    pub last_activity: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Player(Address),
    PlayerCount,
    Leaderboard,
}

#[contract]
pub struct PlayerContract;

#[contractimpl]
impl PlayerContract {
    //register new player
    pub fn register_player(env: Env, player: Address, username: String) -> Result<(), &'static str> {
        // check if player is already registered
        if env.storage().instance().has(&DataKey::Player(player.clone())) {
            return Err("Player already registered");
        }

        let new_player = Player {
            address: player.clone(),
            username: username.clone(),
            level: 1,
            experience: 0,
            total_mined: 0,
            active_mines: Vec::new(&env),
            last_activity: env.ledger().timestamp(),
        };

        // save player
        env.storage().instance().set(&DataKey::Player(player.clone()), &new_player);
        
        // update total player count
        let mut player_count: u32 = env.storage().instance()
            .get(&DataKey::PlayerCount)
            .unwrap_or(0);
        player_count += 1;
        env.storage().instance().set(&DataKey::PlayerCount, &player_count);

        Ok(())
    }

    // get player data
    pub fn get_player(env: Env, player: Address) -> Option<Player> {
        env.storage().instance().get(&DataKey::Player(player))
    }

    // update player experience
    pub fn update_experience(env: Env, player: Address, exp_gained: u64) -> Result<(), &'static str> {
        let mut player_data = env.storage().instance()
            .get(&DataKey::Player(player.clone()))
            .ok_or("Player not found")?;

        player_data.experience += exp_gained;
        player_data.last_activity = env.ledger().timestamp();
        
        // calculate level (simple formula: every 1000 exp = 1 level)
        let new_level = (player_data.experience / 1000) + 1;
        if new_level > player_data.level {
            player_data.level = new_level as u32;
        }

        env.storage().instance().set(&DataKey::Player(player), &player_data);
        Ok(())
    }

    // add active mine
    pub fn add_active_mine(env: Env, player: Address, mine_id: u32) -> Result<(), &'static str> {
        let mut player_data = env.storage().instance()
            .get(&DataKey::Player(player.clone()))
            .ok_or("Player not found")?;

        player_data.active_mines.push_back(mine_id);
        player_data.last_activity = env.ledger().timestamp();

        env.storage().instance().set(&DataKey::Player(player), &player_data);
        Ok(())
    }

    // update total mining amount
    pub fn update_total_mined(env: Env, player: Address, amount: u64) -> Result<(), &'static str> {
        let mut player_data = env.storage().instance()
            .get(&DataKey::Player(player.clone()))
            .ok_or("Player not found")?;

        player_data.total_mined += amount;
        player_data.last_activity = env.ledger().timestamp();

        env.storage().instance().set(&DataKey::Player(player), &player_data);
        Ok(())
    }

    // get total player count
    pub fn get_player_count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::PlayerCount).unwrap_or(0)
    }

    // Is player active? (Last 24 hours)
    pub fn is_player_active(env: Env, player: Address) -> bool {
        if let Some(player_data) = env.storage().instance().get(&DataKey::Player(player)) {
            let current_time = env.ledger().timestamp();
            let day_in_seconds = 24 * 60 * 60;
            return current_time - player_data.last_activity < day_in_seconds;
        }
        false
    }
}