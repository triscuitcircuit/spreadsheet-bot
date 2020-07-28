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
use crate::{models, DbPool, Bans, CrossRole};
#[command]
#[description = "role add"]
#[example = "@user 2020-12-02"]
#[required_permissions("ADMINISTRATOR")]
#[only_in(guilds)]
#[owner_privilege]
fn rolelist(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut response = MessageBuilder::new();

    let mut data = ctx.data.write();
    let roles = data.get::<CrossRole>().unwrap();
    let db = data.get::<DbPool>().unwrap();

    roles.iter().for_each(|(x,y)|{
        response.push(format!("roleid: {} {}",x.0,y[0].get_id())) ;
    });
    if let Err(e) = msg.channel_id.say(&ctx,response){
        println!("error sending message `{}`\n",e);
    };


    Ok(())
}

#[command]
#[description = "role add"]
#[example = "@user 2020-12-02"]
#[required_permissions("ADMINISTRATOR")]
#[only_in(guilds)]
#[owner_privilege]
fn rolelistadd(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();

    let (discord_role, new_role)={
      let db = data.get::<DbPool>().unwrap();

        let role_id = args.single::<RoleId>();
        let time = args.single::<String>();
        let global = args.single::<bool>();

        let (discord_role, user)= match role_id {
            Ok(role_id)=>{
                let discord_role = role_id.to_role_cached(&ctx).unwrap();
                let user = models::Role::get(role_id,&db);
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