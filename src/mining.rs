#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetalType { // Types of metals can be mined
    Gold,
    Silver,
    Copper,
    Iron,
    Platinum,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mine {
    pub id: u32,
    pub owner: Address,
    pub metal_type: MetalType,
    pub efficiency: u32,        // between1-100 
    pub capacity: u64,          // Maximum production capacity
    pub current_production: u64, // Curren productıon
    pub start_time: u64,        // Mining start time
    pub last_harvest: u64,      // Last harvest time
    pub upgrade_level: u32,     // Mine level
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinedResource {
    pub metal_type: MetalType,
    pub amount: u64,
    pub mined_at: u64,
    pub efficiency_bonus: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Mine(u32),
    PlayerMines(Address),
    MineCount,
    GlobalProduction(MetalType),
}

#[contract]
pub struct MiningContract;

#[contractimpl]
impl MiningContract {
    // new mine creation
    pub fn create_mine(
        env: Env, 
        owner: Address, 
        metal_type: MetalType
    ) -> Result<u32, &'static str> {
        owner.require_auth();

        let mine_count: u32 = env.storage().instance()
            .get(&DataKey::MineCount)
            .unwrap_or(0);
        
        let mine_id = mine_count + 1;
        
        let new_mine = Mine {
            id: mine_id,
            owner: owner.clone(),
            metal_type: metal_type.clone(),
            efficiency: Self::calculate_base_efficiency(&metal_type),
            capacity: Self::calculate_base_capacity(&metal_type),
            current_production: 0,
            start_time: env.ledger().timestamp(),
            last_harvest: env.ledger().timestamp(),
            upgrade_level: 1,
        };

        // save mine
        env.storage().instance().set(&DataKey::Mine(mine_id), &new_mine);
        
        // update players mine list
        let mut player_mines: Vec<u32> = env.storage().instance()
            .get(&DataKey::PlayerMines(owner.clone()))
            .unwrap_or(Vec::new(&env));
        player_mines.push_back(mine_id);
        env.storage().instance().set(&DataKey::PlayerMines(owner), &player_mines);
        
        // Update total mine count
        env.storage().instance().set(&DataKey::MineCount, &mine_id);

        Ok(mine_id)
    }

    /// get mine data
    pub fn get_mine(env: Env, mine_id: u32) -> Option<Mine> {
        env.storage().instance().get(&DataKey::Mine(mine_id))
    }

    /// get player mines
    pub fn get_player_mines(env: Env, player: Address) -> Vec<u32> {
        env.storage().instance()
            .get(&DataKey::PlayerMines(player))
            .unwrap_or(Vec::new(&env))
    }

    /// mining
    pub fn harvest_mine(env: Env, mine_id: u32) -> Result<MinedResource, &'static str> {
        let mut mine = env.storage().instance()
            .get(&DataKey::Mine(mine_id))
            .ok_or("Mine not found")?;

        mine.owner.require_auth();

        let current_time = env.ledger().timestamp();
        let time_since_last_harvest = current_time - mine.last_harvest;
        
        // has 1 hour passed?
        if time_since_last_harvest < 3600 {
            return Err("Must wait at least 1 hour between harvests");
        }

        // calculating procution amount (1 hour)
        let hours_passed = time_since_last_harvest / 3600;
        let base_production = Self::calculate_production_rate(&mine.metal_type, mine.upgrade_level);
        let efficiency_multiplier = mine.efficiency as u64;
        
        let produced_amount = (base_production * hours_passed * efficiency_multiplier) / 100;
        let final_amount = if produced_amount > mine.capacity {
            mine.capacity
        } else {
            produced_amount
        };

        // update mine
        mine.current_production += final_amount;
        mine.last_harvest = current_time;
        env.storage().instance().set(&DataKey::Mine(mine_id), &mine);

        // update global production
        let mut global_production: u64 = env.storage().instance()
            .get(&DataKey::GlobalProduction(mine.metal_type.clone()))
            .unwrap_or(0);
        global_production += final_amount;
        env.storage().instance().set(
            &DataKey::GlobalProduction(mine.metal_type.clone()), 
            &global_production
        );

        let mined_resource = MinedResource {
            metal_type: mine.metal_type.clone(),
            amount: final_amount,
            mined_at: current_time,
            efficiency_bonus: mine.efficiency,
        };

        Ok(mined_resource)
    }

    /// upgrade mine
    pub fn upgrade_mine(env: Env, mine_id: u32) -> Result<(), &'static str> {
        let mut mine = env.storage().instance()
            .get(&DataKey::Mine(mine_id))
            .ok_or("Mine not found")?;

        mine.owner.require_auth();

        if mine.upgrade_level >= 10 {
            return Err("Maximum upgrade level reached");
        }

        mine.upgrade_level += 1;
        mine.efficiency += 5; // Her seviyede %5 verimlilik artışı
        mine.capacity += Self::calculate_base_capacity(&mine.metal_type) / 10; // %10 kapasite artışı

        env.storage().instance().set(&DataKey::Mine(mine_id), &mine);
        Ok(())
    }

    /// get global production data
    pub fn get_global_production(env: Env, metal_type: MetalType) -> u64 {
        env.storage().instance()
            .get(&DataKey::GlobalProduction(metal_type))
            .unwrap_or(0)
    }

    // Helper Functions
    fn calculate_base_efficiency(metal_type: &MetalType) -> u32 {
        match metal_type {
            MetalType::Iron => 80,
            MetalType::Copper => 75,
            MetalType::Silver => 60,
            MetalType::Gold => 45,
            MetalType::Platinum => 30,
        }
    }

    fn calculate_base_capacity(metal_type: &MetalType) -> u64 {
        match metal_type {
            MetalType::Iron => 1000,
            MetalType::Copper => 800,
            MetalType::Silver => 500,
            MetalType::Gold => 200,
            MetalType::Platinum => 100,
        }
    }

    fn calculate_production_rate(metal_type: &MetalType, upgrade_level: u32) -> u64 {
        let base_rate = match metal_type {
            MetalType::Iron => 50,
            MetalType::Copper => 40,
            MetalType::Silver => 25,
            MetalType::Gold => 10,
            MetalType::Platinum => 5,
        };
        base_rate * upgrade_level as u64
    }
}