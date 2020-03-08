pub mod spreadsheet;

#[macro_use]

extern crate lazy_static;
extern crate yard;

use serenity::client::Client;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use serenity::model::channel::{Message, Embed};
use serenity::prelude::{EventHandler, Context};
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};
use serenity::CacheAndHttp;
use std::env;
use std::thread;
use serenity::client::validate_token;
use serenity::utils::MessageBuilder;
use serenity::model::gateway::Activity;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serenity::model::guild::{Guild, Member};
use serenity::http::routing::RouteInfo::CreateMessage;
use serenity::builder::CreateEmbed;
use std::collections::HashMap;
use serenity::model::id::UserId;

struct Handler;
impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {

        let activity = "use 'help;' for spreadsheet commands";
        ctx.set_activity(Activity::playing(&activity));
        if (msg.content.starts_with(";")|| msg.content.ends_with(";")) && msg.content.len() > 1 {
            let input = &msg.content.replace(";","");
            match input.to_uppercase().as_ref(){
                "SERVERS"=>{
                    let mut input_arr:Vec<String> = input.split_whitespace().map(|x| x.to_string()).collect();
                    let string = ctx.clone();
                    let test = &string.cache.read().guilds;
                    let mut trt:String = "".to_string();
                    if input_arr.len() >= 2{
                        let c : u8 = match input_arr[1].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>() {
                            Ok(output) => output,
                            Err(_e) => 0,
                        };
                        let mut i = 0;
                        let mut member_map= HashMap::new();
                        for val in test{
                            member_map = val.1.read().members.clone();
                            if i >= c{
                             break;
                            }else{
                                i += 1;
                            }
                        }
                        for val in member_map{
                            trt = format!("{}\n> {}", trt, val.1.user.read().name);
                        }
                        if let Err(why) =  msg.reply(ctx,format!("{}",trt)){
                            println!("Error sending message: {:?}",why);
                        };
                    }else{
                        for val in test{
                            trt = format!("{}\n> {}", trt, val.1.read().name);
                        }
                        println!("{}",trt);

                        if let Err(why) =  msg.reply(ctx,format!("{}",trt)){
                            println!("Error sending message: {:?}",why);
                        };
                    }
                }
                "HELP"=>{
                    let url = "https://discordapp.com/api/oauth2/authorize?client_id=684150439721304095&permissions=0&scope=bot";
                    let help = format!(">>> Spreadsheet-bot command basics:\n\
                     -Every command for spreadsheet-bot  starts with the prefix `;` followed by a cell to reference on the sheet\n\
                     -A reference to a cell is done by the column letter followed by row number (ex: `a1`)\n\
                     -A cell can be set by a cell reference followed by a equal sign ( separated by a space ) (ex: `a1  = 2`)\n\
                     -A cell can be set to a string, instead of a number, when quotes are in place ( ex: `a1 = \"hello world\" `)\n\
                     -A cell could also reference other cells by putting a cell reference in the deceleration (ex: `a1 = ( b1 * 2 )` )\n\
                     they can also reference multiple cells\n\n\
                     -Spreadsheet can be printed with `;spread`, `;spreadsheet` ,or `;print`\n\
                     -Spreadsheet can be cleared with the `;clear` command, or combined with a cell ref to clear a cell (ex: `;clear a1`)\n\n\
                     The spreadsheet is the same for every server that it is on and can be changed by anyone\n\
                     Creator: ***Chilla#4568***\n\
                      invite the bot with this link: {}",url);
                    if let Err(why) = msg.author.direct_message(ctx,|ret|{
                        ret.embed(|r|
                            r.description(&help).color((0,255,0))

                        );
                        ret
                    }){
                        println!("Error sending message: {:?}",why);
                    };
                }
                "CREDIT"=>{
                    let response = MessageBuilder::new()
                        .push_quote_line("Spreadsheet bot creator: Chilla#4568")
                        .push_quote_line("Discord bot API credit: Serenity Team");
                    //msg.channel_id.broadcast_typing(ctx);
                    if let Err(why) =msg.channel_id.say(&ctx.http,&response){
                        println!("Error sending message: {:?}",why);
                    };
                }
                _=>{
                    let mut  l = spreadsheet::enter_command(input.parse().unwrap());

                    println!("username:{},command:{}",msg.author.name,msg.content);
                    println!("user id:{}, username:{}, spreadsheet \n{}",msg.author.id,msg.author.name,l);
                    l = format!("\n```{}```",l);
                    if let Err(why) =  msg.reply(ctx,l ){
                        println!("Error sending message: {:?}",why);
                    };
                }
            }
        }
    }
}


fn main() {

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Err creating client");
    if let Err(why) = client.start_shards(2) {
        println!("Client error: {:?}", why);
    }
}
