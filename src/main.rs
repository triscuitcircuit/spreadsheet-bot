#[macro_use]extern crate lazy_static;
#[macro_use]extern crate yard;
#[macro_use]extern crate csv;
#[macro_use]extern crate diesel;
#[macro_use]extern crate rand;

use typemap::Key;

use commands::lock::*;

mod commands;
pub mod models;
pub mod schema;
use std::{
    {env,thread},
    sync::{Arc,Mutex},
    time::{Duration,SystemTime},
    collections::{HashMap,HashSet},
    io::Read
};
use serenity::{
  client::Client,
  CacheAndHttp,
  http::{self,client::Http,routing::RouteInfo::CreateMessage},
  client::{validate_token,bridge::gateway::ShardManager},
  model::{gateway::{Activity, Ready},
          guild::{Guild, Member},id::UserId,
          channel::{Message, Embed}
         },
  utils::MessageBuilder,
  builder::CreateEmbed,
};

use serde::{Serialize, Deserialize};
use serenity::prelude::{EventHandler, Context, TypeMapKey};
use serenity::framework::standard::{StandardFramework, CommandResult, macros::{
    command,
    group,
    check
}, HelpOptions, Args, CommandGroup, help_commands, CommandOptions, CheckResult, DispatchError};
use commands::{
    bot_commands::*,
};
use serenity::model::event::ResumedEvent;
use std::path::Path;
use diesel::{
    SqliteConnection,
    r2d2::{ ConnectionManager, Pool },
};
use std::borrow::BorrowMut;
use std::io::Error;

struct CommandCounter;
impl TypeMapKey for CommandCounter{
    type Value = HashMap<String,u64>;
}
struct ShardManagerContainer;
impl Key for ShardManagerContainer {
    type Value = Arc<serenity::prelude::Mutex<ShardManager>>;
}

pub enum Usernum{
    Userdata{username: String,url: String}
}

lazy_static! {
    pub static ref USERS: Mutex<Vec<String>> = Mutex::new(vec!["Nobody".to_string();3]);
}


pub type DbPoolType = Arc<Pool<ConnectionManager<SqliteConnection>>>;
pub struct DbPool(DbPoolType);

impl Key for DbPool{
    type Value = DbPoolType;
}
struct Bans;
impl Key for Bans{
    type Value = HashMap<serenity::model::prelude::UserId,Vec<models::Ban>>;
}


struct Handler;
impl EventHandler for Handler {

    fn ready(&self,ctx:Context,ready: Ready){

        let mut  a = Usernum::Userdata {username: "".to_string(),url:"".to_string()};
        //set_game_presence_help(&ctx);
        let ctx = Arc::new(Mutex::new(ctx));
        if let Some(shard) = ready.shard {
            match shard[0] {
                0 => {

                    println!("Connected as {}", ready.user.name);
                },
                1 => {
                    println!("{}","thread active");
                    status_thread(ready.user.id, ctx)},
                _ => { },
            };

            println!(
                "{} is connected on shard {}/{}!",
                ready.user.name,
                shard[0],
                shard[1],
            );
        }
    }
    fn resume(&self,_:Context,_:ResumedEvent){
        println!("Resumed");
    }
}
fn set_game_presence(ctx: &Context, game_name: &str) {
    let game = serenity::model::gateway::Activity::playing(game_name);
    let status = serenity::model::user::OnlineStatus::Online;
    ctx.set_presence(Some(game), status);
}
fn set_game_presence_help(ctx: &Context) {
    let prefix = String::from(";");
    set_game_presence(ctx, &format!("Type {}sh for spreadsheet help", prefix));
}

fn get_guilds(ctx: &Context) -> Result<usize, serenity::Error> {
    Ok(*&ctx.cache.read().guilds.len().clone() as usize)
}
fn status_thread(user_id:UserId, ctx: Arc<Mutex<Context>>){
    std::thread::spawn(move||
        loop{
            set_game_presence_help(&ctx.lock().unwrap());
            std::thread::sleep(std::time::Duration::from_secs(15));
            let guilds = get_guilds(&ctx.lock().unwrap());//TODO errors out here
            match guilds{
                Ok(count)=>{
                    set_game_presence(&ctx.lock().unwrap(),&format!("Excelling {} servers",count));
                    std::thread::sleep(std::time::Duration::from_secs(18));
                },
                Err(e) => println!("Error while retrieving guild count: {}", e),
            }
            set_game_presence(&ctx.lock().unwrap(),&format!("Use ;help for command list"));
            std::thread::sleep(std::time::Duration::from_secs(13));


        }
    );
}
#[check]
#[name = "Admin"]
// Whether the check shall be tested in the help-system.
#[check_in_help(true)]
// Whether the check shall be displayed in the help-system.
#[display_in_help(true)]
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {
        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }

    false.into()
}


#[group]
#[commands(servers,config,lock,unlock)]
#[checks(Admin)]
#[description = ":star: Administrator"]
struct Owners;

#[group]
#[commands(ping,about,telephone,curtime,telelink)]
#[description = ":clipboard: About"]
struct General;

#[group]
#[commands(spread,invite,spreadsheethelp,export)]
#[description = ":bar_chart: Spreadsheet"]
struct Spreadsheet;

#[derive(Debug)]
enum UserError{
    MutexError,
}

fn set_data()->Result<(),UserError>{
    let mut db = USERS.lock().map_err(|_|UserError::MutexError)?;
    db[1] = "https://discordapp.com/assets/dd4dbc0016779df1378e7812eabaa04d.png".to_string();
    db[2] = "No time stated".to_string();
    Ok(())
}

fn main() {
    if let Err(e) = set_data(){
        panic!("Error: {:?}",e);
    };
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Err creating client");
    {
        let mut data = client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }
    let owners = match client.cache_and_http.http.get_current_application_info(){
        Ok(info)=>{
            let mut set = HashSet::new();
            set.insert(info.owner.id);
            set
        },
        Err(why)=> panic!("Couldn't get application info: {:?}", why),

    };
    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .owners(owners)
            .prefix(";"))
        .help(&SPREADSHEETBOT_HELP)
        .group(&GENERAL_GROUP)
        .group(&OWNERS_GROUP)
        .group(&SPREADSHEET_GROUP)
        .on_dispatch_error(|ctx,msg,error|{
         match error{
             DispatchError::Ratelimited(seconds)=>{
                 if let Err(e) = msg.reply(ctx,&format!("Try command again in {} seconds",seconds)){
                     println!("Error trying to send command {}",e);
                 };
             },
             DispatchError::OnlyForOwners | DispatchError::LackingPermissions(_)|DispatchError::LackingRole|DispatchError::BlockedUser =>{
               if let Err(e) = msg.reply(ctx,"you're not allowed to do this"){
                 println!("Error sending message {}",e);
               };
             },
             DispatchError::BlockedGuild=>{
                 if let Err(e) = msg.reply(ctx,"not available on the server"){
                     println!("Error sending message {}",e);
                 }
             }
             _ => {}
         }
        }));
    let shard_manager = client.shard_manager.clone();
    std::thread::spawn(move||{
        loop {
            std::thread::sleep(std::time::Duration::from_secs(600));

            let lock = shard_manager.lock();
            let shard_runners = lock.runners.lock();

            for (id, runner) in shard_runners.iter() {
                println!(
                    "Shard ID {} is {} with a latency of {:?}",
                    id,
                    runner.stage,
                    runner.latency,
                );
            }
        }
    });
    if let Err(why) = client.start_shards(2) {
        println!("Client error: {:?}", why);
    }

    let http_client = Http::new_with_token(&token);
}
