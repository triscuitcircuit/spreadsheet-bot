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
struct General;
use std::env;
use std::thread;
use serenity::client::validate_token;
use serenity::utils::MessageBuilder;
use serenity::model::gateway::Activity;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use serenity::model::guild::Guild;

struct Handler;

lazy_static!{
    static ref ARRAY:Mutex<Vec<InputInfo>> = Mutex::new(vec![InputInfo{
        user: "".to_string(),
        input: "".to_string(),
        time: SystemTime::now()
    }; 2]);
}

#[derive(Debug, Clone,Deserialize,Serialize)]
struct InputInfo{
    user: String,
    input: String,
    time: SystemTime
}
impl InputInfo{
    pub fn new(user: String, input:String)->InputInfo{
        self::InputInfo{
            user,
            input,
            time: SystemTime::now()
        }
    }
    pub fn get_info(self)-> String{
        format!("last user: {}, entered command: {}, at time {:#?}",self.user,self.input,self.time)
    }
    fn add_elem(&mut self, input:InputInfo){

    }
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {

        let activity = "use 'help;' for spreadsheet commands";

        // let arc = Arc::new(&ctx.shard);
        // let a = thread::spawn(move||{
        //     loop {
        //         thread::sleep(Duration::from_secs(15));
        //         for i in 0..=activity.len()-5 {
        //             arc.set_activity(Option::from(Activity::playing(activity[i..i + 5].as_ref())));
        //         }
        //     }
        // });
//test
        if (msg.content.starts_with(";")|| msg.content.ends_with(";")) && msg.content.len() > 1 {
            let input = &msg.content.replace(";","");
            match input.to_uppercase().as_ref(){
                "SERVERS"=>{
                    let string = ctx.clone();
                    let test = &string.cache.read().guilds;
                    let mut trt:String = "".to_string();
                    for val in test{
                        trt = format!("{}\n> {}", trt, val.1.read().name);
                    }
                    println!("{}",trt);

                    if let Err(why) =  msg.reply(ctx,format!("{}",trt)){
                        println!("Error sending message: {:?}",why);
                    };

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
                    if let Err(why) =msg.reply(ctx,"```Spreadsheet bot creator: Chilla#4568\nDiscord bot API credit: Serenity Team```"){
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
