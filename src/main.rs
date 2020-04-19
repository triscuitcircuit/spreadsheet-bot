mod commands;
#[macro_use]

extern crate lazy_static;
extern crate yard;
extern crate typemap;

use typemap::Key;

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
  client::validate_token,
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

#[derive(Default, Deserialize, Clone)]
pub struct Settings {
    pub discord_token: String,
    pub dbl_api_key: Option<String>,
    pub command_prefix: String,
    pub bot_owners: Vec<serenity::model::prelude::UserId>,

}
impl Key for Settings {
    type Value = Arc<Mutex<Settings>>;
}


struct CommandCounter;
impl TypeMapKey for CommandCounter{
    type Value = HashMap<String,u64>;
}

struct Handler;
impl EventHandler for Handler {
    fn ready(&self,ctx:Context,ready: Ready){
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
fn get_command_prefix(ctx: &Context) -> String {
    let data = ctx.data.read();
    let settings = data.get::<Settings>().unwrap().lock().unwrap();
    settings.command_prefix.clone()
}


fn get_guilds(ctx: &Context) -> Result<usize, serenity::Error> {
    let mut count = 0;
    let string = ctx.clone();
    let test = &string.cache.read().guilds;
    for _val in test{
        count = count + 1;
    }
    Ok(count)
}
fn status_thread(user_id:UserId, ctx: Arc<Mutex<Context>>){
    // let dbl_api_key = {
    //     let ctx = ctx.lock().unwrap();
    //     let data = ctx.data.read();
    //     let settings = data.get::<Settings>().unwrap().lock().unwrap();
    //     settings.dbl_api_key.clone()
    // };
    //not needed right now
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
#[commands(servers)]
#[checks(Admin)]
#[description = ":star: Administrator"]
struct Owners;

#[group]
#[commands(ping,about)]
#[description = ":clipboard: About"]
struct General;

#[group]
#[commands(spread,invite,spreadsheethelp)]
#[description = ":bar_chart: Spreadsheet"]
struct Spreadsheet;


fn main() {

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
        );
    let shard_manager = client.shard_manager.clone();
    std::thread::spawn(move||{
        loop {
            std::thread::sleep(std::time::Duration::from_secs(30));

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
