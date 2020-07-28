use chrono::prelude::{ NaiveDate, NaiveDateTime };
use serenity::{
    prelude::Context,
    model::{
        channel::Message,
        prelude::{ UserId, Permissions },
    },
    framework::standard::{ Args, CommandResult, macros::command, ArgError::Parse },
};

use std::{
    collections::hash_map::Entry::{ Vacant, Occupied },
    sync::Mutex
};

use crate::{ models, DbPool, Bans };

#[command]
#[description = "lock a user from bot use"]
#[example = "@user 2020-12-02"]
#[required_permissions("ADMINISTRATOR")]
#[only_in(guilds)]
#[owner_privilege]
fn lock(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();

    let (discord_user, new_ban) = {
        let db = data.get::<DbPool>().unwrap();

        let user_id = args.single::<UserId>();
        let time = args.single::<String>();
        let global = args.single::<bool>();
        println!("{:#?}",user_id);

        let (discord_user, user) = match user_id {
            Ok(user_id) => {
                let discord_user = user_id.to_user(&ctx).unwrap();
                let user = models::User::get(user_id, &db);
                (discord_user, user)
            },
            Err(Parse(e)) => {
                let _ = msg.reply(&ctx, &format!("Please specify a valid user to lock ({}).", e));
                return Ok(());
            },
            Err(_e) => {
                let _ = msg.reply(&ctx, "Please specify the user to lock.");
                return Ok(());
            },
        };
        let (is_bot_owner, is_target_owner) = {
            let settings = vec![UserId(330181520369254400);1];
            let owners = &settings;
            (
                owners.contains(&msg.author.id),
                owners.contains(&discord_user.id),
            )
        };

        if discord_user.id == msg.author.id {
            let _ = msg.reply(&ctx, "You cannot ban yourself...");
            return Ok(());
        }

        if is_target_owner {
            let _ = msg.reply(&ctx, "You cannot ban this user.");
            return Ok(());
        }

        let channel = msg.channel_id;
        let channel = channel.to_channel(&ctx);
        let channel = match channel {
            Ok(channel) => channel,
            Err(e) => {
                println!("ban.rs: Could not fetch channel: {}", e);
                let _ = msg.reply(&ctx, &format!("An error occurred ({})", e));
                return Ok(());
            },
        };

        let is_target_admin = match channel.guild() {
            Some(guild_channel_lock) => {
                let guild = guild_channel_lock.read().guild(&ctx);
                match guild {
                    Some(guild_lock) => {
                        let permissions = guild_lock.read().member_permissions(discord_user.id);
                        permissions.contains(Permissions::ADMINISTRATOR)
                    },
                    None => false,
                }
            },
            None => false,
        };

        if !is_bot_owner && is_target_admin {
            let _ = msg.reply(&ctx, "You cannot lock an other guild administrator.");
            return Ok(());
        }

        let guild = match global {
            Ok(global) => {
                if global {
                    if !is_bot_owner {
                        let _ = msg.reply(&ctx, "You need to be a bot owner to lock someone globally.");
                        return Ok(());
                    }
                    None
                } else {
                    msg.guild_id
                }
            },
            Err(_) => msg.guild_id,
        };
        let is_already_banned = {
            let bans = data.get::<Bans>().unwrap();
            bans.iter().any(| (user_id, bans_for_user) | {
                *user_id == discord_user.id && bans_for_user.iter().any(| b | {
                    b.is_banned_for_guild(msg.guild_id) && !b.is_over()
                })
            })
        };
        if is_already_banned {
            let _ = msg.reply(&ctx, "This user is already banned.");
            return Ok(());
        }

        let time = match time {
            Ok(mut time) => {
                time = time.trim().into();
                time = time.to_lowercase();
                if time == "permanent" || time == "infinite" || time == "forever" {
                    None
                } else {
                    match NaiveDateTime::parse_from_str(&time, "%Y-%m-%d-%H:%M") {
                        Ok(time) => Some(time),
                        Err(_) => {
                            match NaiveDate::parse_from_str(&time, "%Y-%m-%d") {
                                Ok(date) => Some(date.and_hms(0, 0, 0)),
                                Err(_) => {
                                    let _ = msg.reply(&ctx, "Invalid lock end time. Please use the format \"yyyy-mm-dd[-hh:mm]\"");
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            },
            Err(_) => None,
        };

        let new_ban = user.ban(&db, time, guild);
        (discord_user, new_ban)
    };

    let bans = data.get_mut::<Bans>().unwrap();
    let vec = match bans.entry(discord_user.id) {
        Vacant(entry) => entry.insert(Vec::new()),
        Occupied(entry) => entry.into_mut(),
    };
    vec.push(new_ban);

    let _ = msg.reply(&ctx, &format!("<:banhammer:525343781441110017> locked {}!", discord_user));

    Ok(())
}
#[command]
#[description = "lifts previous user bot lock"]
#[example = "@user"]
#[required_permissions("ADMINISTRATOR")]
#[only_in(guilds)]
#[owner_privilege]
fn unlock(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let db = data.get::<DbPool>().unwrap();
    let user_id = args.single::<UserId>();
    let global = args.single::<bool>();

    let (discord_user, user) = match user_id {
        Ok(user_id) => {
            let discord_user = user_id.to_user(&ctx).unwrap();
            let user = models::User::get(user_id, &db);
            (discord_user, user)
        },
        Err(Parse(e)) => {
            let _ = msg.reply(&ctx, &format!("Please specify a valid user to unlock ({}).", e))?;
            return Ok(());
        },
        Err(_e) => {
            let _ = msg.reply(&ctx, "Please specify the user to unlock.")?;
            return Ok(());
        },
    };
    let global = match global {
        Ok(global) => global,
        Err(_) => false,
    };

    let (is_banned, is_banned_globally) = {
        let bans = data.get::<Bans>().unwrap();
        let is_banned = bans.iter().any(| (user_id, bans_for_user) | {
            *user_id == discord_user.id && bans_for_user.iter().any(| b | {
                b.is_banned_for_guild(msg.guild_id) && !b.is_over()
            })
        });
        let is_banned_globally = bans.iter().any(| (user_id, bans_for_user) | {
            *user_id == discord_user.id && bans_for_user.iter().all(| b | {
                !b.is_over() && b.is_global()
            })
        });
        (is_banned, is_banned_globally)
    };
    if !is_banned {
        let _ = msg.reply(&ctx, "This user is not locked.")?;
        return Ok(());
    }

    let is_bot_owner = {
        let settings = vec![UserId(330181520369254400);1];
        let owners = &settings;
        owners.contains(&msg.author.id)
    };
    if is_banned_globally && !is_bot_owner {
        let _ = msg.reply(&ctx, "You need to be a bot owner to lift a global lock.")?;
        return Ok(());
    }

    let lifted_ban_id = user.unban(msg.guild_id.unwrap(), (is_bot_owner && global) || (is_bot_owner && is_banned_globally), &db);
    match lifted_ban_id {
        Some(id) => {
            let bans = data.get_mut::<Bans>().unwrap();
            match bans.get_mut(&discord_user.id) {
                Some(bans) => bans.retain(|ban| ban.get_id() != id),
                None => {},
            };
            msg.reply(&ctx, &format!("Successfully unlocked {}!", discord_user))?
        },
        None => msg.reply(&ctx, &format!("Could not find lock entry for {} in database.", discord_user))?,
    };

    Ok(())
}

