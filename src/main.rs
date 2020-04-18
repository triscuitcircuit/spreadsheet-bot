mod commands;

#[macro_use]

extern crate lazy_static;
extern crate yard;

use serenity::http::{self,client::Http};
use serenity::client::Client;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use serenity::model::channel::{Message, Embed};
use serenity::prelude::{EventHandler, Context, TypeMapKey};
use serenity::framework::standard::{StandardFramework, CommandResult, macros::{
    command,
    group
}, HelpOptions, Args, CommandGroup, help_commands};
use serenity::CacheAndHttp;
use std::env;
use std::thread;
use serenity::client::validate_token;
use serenity::utils::MessageBuilder;
use serenity::model::gateway::{Activity, Ready};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serenity::model::guild::{Guild, Member};
use serenity::http::routing::RouteInfo::CreateMessage;
use serenity::builder::CreateEmbed;
use std::collections::{HashMap, HashSet};
use serenity::model::id::UserId;


use commands::{
    bot_commands::*,
};
use serenity::model::event::ResumedEvent;
use serenity::cache::Settings;

#[group]
#[commands(servers,ping,about)]
struct General;

// #[group]
// #[commands(help)]
// struct Help;

// #[command]
// #[wrong_channel = "Strike"]
// #[lacking_permissions = "Hide"]
// #[indention_prefix = "+"]
// #[max_levenshtein_distance(3)]
// #[command_not_found_text = "Could not find: `{}`."]
// fn help(context: &mut Context, msg: &Message, args: Args, help_options: &'static HelpOptions,groups: &[&'static CommandGroup],owners: HashSet<UserId>)->CommandResult{
//     help_commands::with_embeds(context, msg, args, &help_options, groups, owners)
// }

#[group]
#[prefixes("s")]
#[commands(spread)]
struct Spread;

struct CommandCounter;
impl TypeMapKey for CommandCounter{
    type Value = HashMap<String,u64>;
}

struct Handler;
impl EventHandler for Handler {
    fn ready(&self,ctx:Context,ready: Ready){
        let activity = "use 'help;' for spreadsheet commands";
        ctx.set_activity(Activity::playing(&activity));
        println!("Connected as {}", ready.user.name);
    }
    fn resume(&self,_:Context,_:ResumedEvent){
        println!("Resumed");
    }
    // fn message(&self, ctx: Context, msg: Message) {
    //     if (msg.content.starts_with(";")|| msg.content.ends_with(";")) && msg.content.len() > 1 {
    //         let input = &msg.content.replace(";","");
    //         let mut input_arr:Vec<String> = input.splitn(2," ").map(|x| x.to_string()).collect();
    //         match input_arr[0].to_uppercase().as_ref(){
    //             "HELP"=>{
    //                 let url = "https://discordapp.com/api/oauth2/authorize?client_id=684150439721304095&permissions=0&scope=bot";
    //                 let help = format!(">>> Spreadsheet-bot command basics:\n\
    //                  -Every command for spreadsheet-bot  starts with the prefix `;` followed by a cell to reference on the sheet\n\
    //                  -A reference to a cell is done by the column letter followed by row number (ex: `a1`)\n\
    //                  -A cell can be set by a cell reference followed by a equal sign ( separated by a space ) (ex: `a1  = 2`)\n\
    //                  -A cell can be set to a string, instead of a number, when quotes are in place ( ex: `a1 = \"hello world\" `)\n\
    //                  -A cell could also reference other cells by putting a cell reference in the deceleration (ex: `a1 = ( b1 * 2 )` )\n\
    //                  they can also reference multiple cells\n\n\
    //                  -Spreadsheet can be printed with `;spread`, `;spreadsheet` ,or `;print`\n\
    //                  -Spreadsheet can be cleared with the `;clear` command, or combined with a cell ref to clear a cell (ex: `;clear a1`)\n\n\
    //                  The spreadsheet is the same for every server that it is on and can be changed by anyone\n\
    //                  Creator: ***Chilla#4568***\n\
    //                   invite the bot with this link: {}",url);
    //                 if let Err(why) = msg.author.direct_message(ctx,|ret|{
    //                     ret.embed(|r|
    //                         r.description(&help).color((0,255,0))
    //
    //                     );
    //                     ret
    //                 }){
}
fn set_game_presence(ctx: &Context, game_name: &str) {
    let game = serenity::model::gateway::Activity::playing(game_name);
    let status = serenity::model::user::OnlineStatus::Online;
    ctx.set_presence(Some(game), status);
}
fn set_game_presence_help(ctx: &Context) {
    let prefix = get_command_prefix(ctx);
    set_game_presence(ctx, &format!("Type {}help to get a list of available commands", prefix));
}

fn get_guilds(ctx: &Context) -> Result<usize, serenity::Error> {
    let mut count = 0;
    let mut last_guild_id = 0;
    loop {
        let guilds = ctx.http.get_guilds(&http::GuildPagination::After(last_guild_id.into()), 100)?;
        let len = guilds.len();
        count += len;
        if len < 100 {
            break;
        }
        if let Some(last) = guilds.last() {
            last_guild_id = *last.id.as_u64();
        }
    }

    Ok(count)
}
fn status_thread(user_id:UserId, ctx: Arc<Mutex<Context>>){
    let dbl_api_key = {
        let ctx = ctx.lock().unwrap();
        let data = ctx.data.read();
        let settings = data.get::<Settings>().unwrap().lock().unwrap();
        settings.dbl_api_key.clone()
    };
    std::thread::spawn(move||
        loop{
            set_game_presence_help(&ctx.lock().unwrap());
            std::thread::skeep(std::time::Duration::from_secs(30));

            let guilds = get_guilds(&ctx.lock().unwrap());
            match guilds{
                Ok(count)=>{
                    set_game_presence(&ctx.lock().unwrap(),&format!("Excelling {} servers",count));
                }
                Err(e) => error!("Error while retrieving count {}",e),
            }


        }
    );
}
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
        .group(&GENERAL_GROUP)
        //.group(&HELP_GROUP)
        .group(&SPREAD_GROUP)
        .before(|ctx, msg, command_name|{
            println!("Got command '{}' by user '{}'", command_name, msg.author.name);
            let mut data = ctx.data.write();
            let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in ShareMap");
            let entry = counter.entry(command_name.to_string()).or_insert(0);
            *entry +=1;
            true
        }));

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
    if let Err(why) = client.start_shards(2) {
        println!("Client error: {:?}", why);
    }
}
