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
            let mut input_arr:Vec<String> = input.splitn(2," ").map(|x| x.to_string()).collect();
            match input_arr[0].to_uppercase().as_ref(){
                "SERVERS"=>{
                    let string = ctx.clone();
                    let test = &string.cache.read().guilds;
                    let mut trt:String = "".to_string();
                    if input_arr.len() >= 2{
                        println!("{:#?}",input_arr);
                        let input_two = &input_arr[1];
                        if &input_two[0..1] == "\"" && &input_two[input_two.len()-1..input_two.len()] == "\""{
                            let server_named = &input_two[1..input_two.len()-1];
                            for (guild,arc) in test{
                                if arc.read().name.eq(server_named){
                                    let mut response = MessageBuilder::new();
                                    for (userid,username) in &arc.read().members{
                                            response.push(format!(" userid:`{}` username:`{}`\n",userid,username.user.read().name));
											println!("{}",format!(" userid:`{}` username:`{}`\n",userid,username.user.read().name));
											for f in &username.roles{
												//response.push(format!("roles: {}\n",f.to_role_cached(&ctx.cache).unwrap().name));
												println!("{}",format!("roles: {}\n",f.to_role_cached(&ctx.cache).unwrap().name));
											}
                                        //test
                                    }
									if let Err(why) =msg.channel_id.say(&ctx.http,&response){
                                            println!("Error sending message: {:?}",why);
                                        };
                                }
                            }
                            // if let Err(why) =  msg.reply(ctx,format!("{}",trt)){
                            //     println!("Error sending message: {:?}",why);
                            // };

                        } else{
                            if let Err(why) =  msg.reply(ctx,format!("{}","``` Error parsing server name, please enter with quotes,")){
                                println!("Error sending message: {:?}",why);
                            };
                        }
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
