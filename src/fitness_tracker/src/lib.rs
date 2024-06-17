#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct User {
    id: u64,
    name: String,
    email: String,
    points: u64,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Activity {
    id: u64,
    user_id: u64,
    r#type: String,
    duration: u64, // duration in minutes
    date: u64,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Challenge {
    id: u64,
    creator_id: u64,
    title: String,
    description: String,
    participants: Vec<u64>,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Follow {
    id: u64,
    follower_id: u64,
    following_id: u64,
    created_at: u64,
}

impl Storable for User {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Activity {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Activity {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Challenge {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Challenge {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Follow {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Follow {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static USERS_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );

    static ACTIVITIES_STORAGE: RefCell<StableBTreeMap<u64, Activity, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))))
    );

    static CHALLENGES_STORAGE: RefCell<StableBTreeMap<u64, Challenge, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))))
    );

    static FOLLOWS_STORAGE: RefCell<StableBTreeMap<u64, Follow, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))))
    );
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct UserPayload {
    name: String,
    email: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct ActivityPayload {
    user_id: u64,
    r#type: String,
    duration: u64,
    date: u64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct ChallengePayload {
    creator_id: u64,
    title: String,
    description: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct FollowPayload {
    follower_id: u64,
    following_id: u64,
}

#[ic_cdk::update]
fn create_user(payload: UserPayload) -> Result<User, String> {
    if payload.name.is_empty() || payload.email.is_empty() {
        return Err("Invalid input: Ensure 'name' and 'email' are provided.".to_string());
    }

    if !payload.email.contains('@') {
        return Err("Invalid input: Ensure 'email' is a valid email address.".to_string());
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let user = User {
        id,
        name: payload.name,
        email: payload.email,
        points: 0,
        created_at: time(),
    };

    let _ = USERS_STORAGE.with(|storage| storage.borrow_mut().insert(id, user.clone()));

    Ok(user)
}

#[ic_cdk::update]
fn create_activity(payload: ActivityPayload) -> Result<Activity, String> {
    USERS_STORAGE.with(|storage| {
        if storage.borrow().get(&payload.user_id).is_none() {
            return Err("User with the given ID does not exist.".to_string());
        }
        Ok(())
    })?;

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let activity = Activity {
        id,
        user_id: payload.user_id,
        r#type: payload.r#type,
        duration: payload.duration,
        date: payload.date,
        created_at: time(),
    };

    let _ = ACTIVITIES_STORAGE.with(|storage| storage.borrow_mut().insert(id, activity.clone()));
    
    Ok(activity)
}

#[ic_cdk::update]
fn create_challenge(payload: ChallengePayload) -> Result<Challenge, String> {
    USERS_STORAGE.with(|storage| {
        if storage.borrow().get(&payload.creator_id).is_none() {
            return Err("User with the given ID does not exist.".to_string());
        }
        Ok(())
    })?;

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let challenge = Challenge {
        id,
        creator_id: payload.creator_id,
        title: payload.title,
        description: payload.description,
        participants: Vec::new(),
        created_at: time(),
    };

    let _ = CHALLENGES_STORAGE.with(|storage| storage.borrow_mut().insert(id, challenge.clone()));

    Ok(challenge)
}

#[ic_cdk::update]
fn join_challenge(challenge_id: u64, user_id: u64) -> Result<Challenge, String> {
    let mut challenge = CHALLENGES_STORAGE.with(|storage| {
        storage.borrow().get(&challenge_id).ok_or_else(|| "Challenge not found.".to_string())
    })?;

    USERS_STORAGE.with(|storage| {
        if storage.borrow().get(&user_id).is_none() {
            return Err("User with the given ID does not exist.".to_string());
        }
        Ok(())
    })?;

    challenge.participants.push(user_id);
    let _ = CHALLENGES_STORAGE.with(|storage| storage.borrow_mut().insert(challenge_id, challenge.clone()));

    Ok(challenge)
}

#[ic_cdk::update]
fn follow_user(payload: FollowPayload) -> Result<Follow, String> {
    USERS_STORAGE.with(|storage| {
        if storage.borrow().get(&payload.follower_id).is_none() {
            return Err("User with the given ID does not exist.".to_string());
        }
        Ok(())
    })?;

    USERS_STORAGE.with(|storage| {
        if storage.borrow().get(&payload.following_id).is_none() {
            return Err("User with the given ID does not exist.".to_string());
        }
        Ok(())
    })?;

    if payload.follower_id == payload.following_id {
        return Err("Invalid input: User cannot follow themselves.".to_string());
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let follow = Follow {
        id,
        follower_id: payload.follower_id,
        following_id: payload.following_id,
        created_at: time(),
    };

    let _ = FOLLOWS_STORAGE.with(|storage| storage.borrow_mut().insert(id, follow.clone()));

    Ok(follow)
}

#[ic_cdk::query]
fn get_users() -> Result<Vec<User>, String> {
    let users = USERS_STORAGE.with(|storage| storage.borrow().iter().map(|(_, user)| user.clone()).collect());
    Ok(users)
}

#[ic_cdk::query]
fn get_activities() -> Result<Vec<Activity>, String> {
    let activities = ACTIVITIES_STORAGE.with(|storage| storage.borrow().iter().map(|(_, activity)| activity.clone()).collect());
    Ok(activities)
}

#[ic_cdk::query]
fn get_user_activities(user_id: u64) -> Result<Vec<Activity>, String> {
    let activities = ACTIVITIES_STORAGE.with(|storage| storage.borrow().iter().filter_map(|(_, activity)| {
        if activity.user_id == user_id {
            Some(activity.clone())
        } else {
            None
        }
    }).collect());
    Ok(activities)
}

#[ic_cdk::query]
fn get_challenges() -> Result<Vec<Challenge>, String> {
    let challenges = CHALLENGES_STORAGE.with(|storage| storage.borrow().iter().map(|(_, challenge)| challenge.clone()).collect());
    Ok(challenges)
}

#[ic_cdk::query]
fn get_user_followers(user_id: u64) -> Result<Vec<Follow>, String> {
    let followers = FOLLOWS_STORAGE.with(|storage| storage.borrow().iter().filter_map(|(_, follow)| {
        if follow.following_id == user_id {
            Some(follow.clone())
        } else {
            None
        }
    }).collect());
    Ok(followers)
}

#[ic_cdk::query]
fn get_leaderboard() -> Result<Vec<User>, String> {
    let mut users: Vec<User> = USERS_STORAGE.with(|storage| storage.borrow().iter().map(|(_, user)| user.clone()).collect());
    users.sort_by(|a, b| b.points.cmp(&a.points));
    Ok(users.into_iter().take(10).collect())
}

// Need this to generate candid
ic_cdk::export_candid!();
