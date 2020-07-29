use chrono::prelude::{ NaiveDate, NaiveDateTime };
use serenity::{
    prelude::Context,
    utils::MessageBuilder,
    model::{
        channel::Message,
        prelude::{ UserId, Permissions },
        guild::Role,
        id::RoleId
    },
    framework::standard::{ Args, CommandResult, macros::command, ArgError::Parse },
};
use std::{
    collections::hash_map::Entry::{ Vacant, Occupied },
    ops::Deref,
    sync::Mutex
};
use crate::{models, DbPool, Bans, CrossRole, commands::{bot_commands::embed, bot_commands::embed::embed_sender}};
use serenity::model::id::GuildId;
use serenity::model::guild::Guild;

#[command]
#[description = "list public shared roles. Use the numbers to pick a role, and users from the old server will be given the role automatically"]
#[example = ";ir Optional(selection number)"]
#[required_permissions("ADMINISTRATOR")]
#[only_in(guilds)]
#[aliases("ir")]
fn interroles(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {

    let number = args.single::<i32>();
    let keep_members = args.single::<String>();
    let mut response = String::new();

    let mut data = ctx.data.write();
    let roles = data.get::<CrossRole>().unwrap();

    let selection = match number{
        Ok(number)=>{
            roles.iter().for_each(|(x,y)|{
               if y[0].get_id() == number{
                   println!("role selected");
                   let createdrole  = msg.guild(&ctx).unwrap().read().create_role(&ctx,|createdrole|{
                       let existing = x.to_role_cached(&ctx).unwrap();

                       createdrole.name(existing.name);
                       createdrole.colour(existing.colour.0 as u64);
                       createdrole.mentionable(existing.mentionable);
                       createdrole.position(existing.position as u8);
                       createdrole.permissions(existing.permissions);
                       createdrole
                   });
                   match createdrole{
                       Ok(newrole)=>{
                           y[0].get_guild().to_guild_cached(&ctx).unwrap().read().members.iter().for_each(|(a,b)|{
                               msg.guild(&ctx).unwrap().read().members.iter().for_each(|(l, s)|{
                                   if l.eq(a){
                                       if b.roles.contains(x){
                                           if let Err(e) = s.to_owned().add_role(&ctx,newrole.id){
                                               let _ = msg.reply(&ctx,"Make sure bot has proper permissions to add roles");
                                               println!("Couldn't add role to user {}",e);

                                           };
                                       }
                                   }
                               });
                           });
                       }
                       Err(e)=>{
                           let _ = msg.reply(&ctx,"Make sure bot has proper permissions");
                           println!("Couldn't create role{}",e);
                       }
                   }
               }
            });
            println!("{}",number);
            println!("role added to server: {}",msg.guild_id.unwrap().to_guild_cached(&ctx).unwrap().read().name)
        }
        Err(e)=>{
            roles.iter().for_each(|(x,y)|{
                response.push_str(&*format!("{}. rolename: `{}` from server`{}`\n",
                                            y[0].get_id(),
                                            x.to_role_cached(&ctx).unwrap().name,
                                            y[0].get_guild().to_guild_cached(&ctx).unwrap().read().name,
                ));
            });
            embed_sender(&mut ctx.clone(), msg, &msg.channel_id, response);
        }
    };

    let db = data.get::<DbPool>().unwrap();
    Ok(())
}

#[command]
#[description = "Add a server role to the global list of roles"]
#[example = ";iradd @newrole"]
#[required_permissions("ADMINISTRATOR")]
#[only_in(guilds)]
#[aliases("iradd")]
fn interrolesadd(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();

    let (discord_role, new_role)={
      let db = data.get::<DbPool>().unwrap();

        let role_id = args.single::<RoleId>();
        let time = args.single::<String>();
        let global = args.single::<bool>();

        let (discord_role, user)= match role_id {
            Ok(role_id)=>{
                let discord_role = role_id.to_role_cached(&ctx).unwrap();
                let user = models::Role::get(role_id,msg.guild_id.unwrap(),&db);
                (discord_role,user)
            }
            Err(Parse(e))=>{
                let _ = msg.reply(&ctx,&format!("Please specify a valid role ({})",e));
                return Ok(())
            }
            Err(e)=>{
                println!("{}",e);
                let _ = msg.reply(&ctx,"please specify a role");
                return Ok(())
            }
        };
        let new_role = user.list_role(&db,msg.guild_id.unwrap(),discord_role.colour,msg.author.id);
        (discord_role,new_role)
    };
    let roles = data.get_mut::<CrossRole>().unwrap();
    let vec = match roles.entry(discord_role.id){
        Vacant(entry)=> entry.insert(Vec::new()),
        Occupied(entry)=> entry.into_mut(),
    };
    vec.push(new_role);

    let _ = msg.reply(&ctx,&format!("role added to table: {}",discord_role));
    Ok(())
}
#[command]
#[description = "list public shared roles. Use the numbers to pick a role, and users from the old server will be given the role automatically"]
#[example = ";irdel"]
#[required_permissions("ADMINISTRATOR")]
#[only_in(guilds)]
#[aliases("irdel")]
#[owner_privilege]
fn interrolesdel(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {

    let listed_role = args.single::<RoleId>();
    let guild = args.single::<u64>();
    let mut data = ctx.data.write();
    let db = data.get::<DbPool>().unwrap();

    let listedinterroles = data.get::<CrossRole>().unwrap();

    let mut response = String::new();

    match listed_role {
        Ok(listedrole)=>{

            let selected_guild = match guild{
                Ok(guild)=>{
                    GuildId::from(guild)
                },
                Err(e)=>{
                    let _ = msg.reply(&ctx, "Please specify the guild to unlist.")?;
                    return Ok(());
                }
            };

            let discord_role = listedrole.to_role_cached(&ctx).unwrap();
            let role = models::Role::get(listedrole, selected_guild,&db);
            let lifted_listid = role.unlist(selected_guild,&db);
            match lifted_listid{
                Some(id)=>{
                    let roles = data.get_mut::<CrossRole>().unwrap();
                    match roles.get_mut(&listedrole){
                        Some(rs)=>{rs.retain(|role| role.get_id() != id)}
                        None=>{}
                    };
                    msg.reply(&ctx, &format!("Successfully removed {}!", discord_role.name))?

                },
                None=>{
                    msg.reply(&ctx, &format!("Could not find lock entry for `{}` in database.", discord_role.name))?
                }
            };

        }
        Err(Parse(e))=>{
            let _ = msg.reply(&ctx, &format!("Please specify a valid role to unlock ({}).", e))?;
            return Ok(());
        }
        Err(e) => {
            listedinterroles.iter().for_each(|(x, y)| {
                response.push_str(&*format!("{}. roleid: `{}` rolename `{}` {} server-name:`{}`server id:`{}`\n",
                                            y[0].get_id(),
                                            x.0,
                                            x.to_role_cached(&ctx).unwrap().name,
                                            x.to_role_cached(&ctx).unwrap(),
                                            y[0].get_guild().to_guild_cached(&ctx).unwrap().read().name,
                                            y[0].get_guild().0
                )
                );
            });
            embed_sender(&mut ctx.clone(), msg, &msg.channel_id, response);
        }
    }
    Ok(())
}
