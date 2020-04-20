use chrono::prelude::NaiveDateTime;
use diesel::prelude::*;
use serenity::model::prelude::{GuildId, UserId};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

use crate::{schema, DbPoolType};

#[derive(Queryable)]
pub struct User {
    id:         i32,
    discord_id: String,
}

#[derive(Queryable, Clone)]
pub struct Ban {
    id:        i32,
    user:      i32,
    guild:     Option<String>,
    end_epoch: Option<String>,
}

#[derive(Queryable)]
pub struct LangStat {
    id:                i32,
    lang_name:         String,
    snippets_executed: i32,
}

#[derive(Queryable)]
#[allow(dead_code)]
pub struct Snippet {
    id:       i32,
    user:     i32,
    code:     String,
    guild:    Option<String>,
    run_time: String,
}

impl User {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_discord_id(&self) -> UserId {
        self.discord_id.parse::<u64>().expect("Could not parse UserId from string").into()
    }

    pub fn get(discord_user_id: UserId, db: &DbPoolType) -> Self {
        use schema::user::dsl::*;

        let discord_user_id = discord_user_id.to_string();

        let db = db.get().unwrap();
        match user.filter(discord_id.eq(&discord_user_id)).first::<User>(&db) {
            Ok(lang) => lang,
            Err(_) => {
                let r = diesel::insert_into(user).values(
                    discord_id.eq(discord_user_id)
                ).execute(&db);
                match r {
                    Ok(_) => {
                        user.order(self.id.desc())
                            .first::<User>(&db)
                            .unwrap()
                    },
                    Err(e) => panic!(e),
                }
            },
        }
    }

    pub fn ban(&self, db: &DbPoolType, ban_end: Option<NaiveDateTime>, ban_on_guild: Option<GuildId>) -> Ban {
        use schema::ban::dsl::*;

        let ban_on_guild = match ban_on_guild {
            Some(ban_on_guild) => Some(format!("{}", ban_on_guild)),
            None => None,
        };
        let ban_end = match ban_end {
            Some(ban_end) => Some(format!("{}", ban_end.timestamp())),
            None => None,
        };

        let db = db.get().unwrap();
        let r = diesel::insert_into(ban).values((
            user.eq(self.id),
            end_epoch.eq(ban_end),
            guild.eq(ban_on_guild),
        )).execute(&db);
        match r {
            Ok(_) => {
                ban.order(self.id.desc())
                    .first::<Ban>(&db)
                    .unwrap()
            },
            Err(e) => panic!(e),
        }
    }

    pub fn unban(&self, msg_guild: GuildId, lift_globally: bool, db: &DbPoolType) -> Option<i32> {
        let db = db.get().unwrap();

        use diesel::dsl::sql;
        use schema::ban::dsl::*;

        let filter = sql(&format!("user = {}", self.id));
        let filter = if !lift_globally {
            filter.sql(&format!(" AND guild = {}", msg_guild))
        } else {
            filter.sql("")
        };
        let ban_id: Option<i32> = ban.select(self.id).filter(&filter).first(&db).ok();

        let _ = diesel::delete(ban)
            .filter(filter)
            .execute(&db);

        ban_id
    }
}

impl Ban {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_user(&self, db: &DbPoolType) -> Option<User> {
        use schema::user::dsl::*;
        let db = db.get().unwrap();
        match user.find(self.user).get_result::<User>(&db) {
            Ok(res) => Some(res),
            Err(_) => None,
        }
    }

    pub fn get_guild(&self) -> Option<GuildId> {
        match self.guild {
            Some(ref guild) => {
                let id = guild.parse::<u64>();
                match id {
                    Ok(id) => Some(id.into()),
                    Err(_) => None,
                }
            },
            None => None,
        }
    }

    pub fn is_permanent(&self) -> bool {
        self.end_epoch.is_none()
    }

    pub fn is_over(&self) -> bool {
        if self.is_permanent() {
            return false;
        }

        let epoch = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => return false,
        };
        let end_epoch = match self.end_epoch.clone().unwrap().parse::<u64>() {
            Ok(n) => n,
            Err(_) => return false,
        };
        end_epoch < epoch
    }

    pub fn cleanup_outdated_bans(db: &DbPoolType) {
        use diesel::dsl::sql;
        use schema::ban::dsl::*;

        let db = db.get().unwrap();
        let epoch = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => return,
        };
        let _ = diesel::delete(ban)
            .filter(end_epoch.is_not_null()
                .and(sql(&format!("end_epoch < {}", epoch))))
            .execute(&db);
    }

    pub fn get_bans(db: &DbPoolType) -> HashMap<UserId, Vec<Ban>> {
        use schema::ban::dsl::*;

        let res = ban.get_results::<Ban>(&db.get().unwrap());
        match res {
            Ok(bans) => {
                let bans: Vec<Ban> = bans.into_iter().filter(| b | !b.is_over()).collect();
                let mut map: HashMap<UserId, Vec<Ban>> = HashMap::new();
                for b in bans {
                    if b.is_over() {
                        continue;
                    }
                    let b_user = b.get_user(&db);
                    let b_user = match b_user {
                        Some(u) => u.get_discord_id(),
                        None => continue,
                    };
                    let vec = match map.entry(b_user) {
                        Vacant(entry) => entry.insert(Vec::new()),
                        Occupied(entry) => entry.into_mut(),
                    };
                    vec.push(b);
                }
                map
            },
            Err(_) => HashMap::new(),
        }
    }

    pub fn is_banned_for_guild(&self, msg_guild: Option<GuildId>) -> bool {
        let guild = self.get_guild();
        !self.is_over() && (guild.is_none() || msg_guild.is_none() || guild.unwrap() == msg_guild.unwrap())
    }

    pub fn is_global(&self) -> bool {
        self.get_guild().is_none()
    }
}
