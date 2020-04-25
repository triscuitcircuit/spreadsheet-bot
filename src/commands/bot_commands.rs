//use "*" to configure bot
use serenity::{
    prelude::*,
    model::{prelude::{UserId,Permissions},channel::{Message,Embed}},
    framework::standard::{
        Args,CommandResult,
        CommandOptions,CommandGroup, DispatchError,
        HelpOptions,help_commands,StandardFramework,
        macros::{command,group,help,check}
    },
    utils::{content_safe,ContentSafeOptions,MessageBuilder},
    client::bridge::gateway::{ShardManager,ShardId}
};
use std::{collections::{HashSet},
          env,fmt::write,
          sync::Arc};
use crate::commands::spreadsheet;
use crate::{models, Bans, DbPool, USERS};
use rand::Rng;
use serenity::http::AttachmentType;
use std::path::Path;
use std::io::Error;

pub(crate) struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer{
    type Value = Arc<Mutex<ShardManager>>;
}
extern crate chrono;
use chrono::{Datelike, Timelike, Utc};
use serenity::http::routing::RouteInfo::CreateMessage;
use serenity::model::id::ChannelId;
use serenity::model::channel::Channel;


#[command]
#[owners_only]
#[description ="get a list of servers that spreadsheet bot is in"]
fn servers(ctx: &mut Context,msg:&Message)->CommandResult{
    let string = ctx.clone();
    let input = &msg.content;
    let mut input_arr:Vec<String> = input.splitn(2," ").map(|x| x.to_string()).collect();
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
                    }
                    if let Err(why) =msg.channel_id.say(&ctx.http,&response){
                        println!("Error sending message: {:?}",why);
                    };
                }
            }
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
    }
    Ok(())
}
#[command]
#[description = "inter server message into another servers channel. Usage ;tl {server_name}{channel_name}{message_content}"]
#[aliases("tl")]
#[only_in(guilds)]
fn telelink(ctx: &mut Context, msg: &Message) -> CommandResult{
    let string = ctx.clone();
    let input: String = {
        if msg.content.contains(";t "){
            String::from(format!("{}",msg.content.replace(";tl ", "")))
        }else{
            String::from(format!("{}",msg.content.replace(";telelink ", "")))
        }
    };
    let input_arr:Vec<String> = input.splitn(3,"}{").map(|x| x.to_string()).collect();
    println!("request sent in for inter-server message :{} , {:?}",&msg.content,input_arr);
    let test = &string.cache.read().guilds;
    let mut a = false;
    if input_arr.len() >= 3{
        let (servername,channelsearch)= (
            &input_arr[0][1..input_arr[0].len()],
            &input_arr[1],
        );
        test.into_iter().for_each(|f|{
            if f.1.read().name.eq(servername){
                a = true;
                let mut x = false;
                let channels = &f.1.as_ref().read().channels;
                channels.into_iter().for_each(|l|{
                    if f.1.read().name.eq(channelsearch){
                        if &input_arr[1].len() -1 == 0{
                            x = true;
                            embed_sender(ctx, msg, l.0, " ".to_string());
                        }else{
                            x= true;
                            embed_sender(ctx,msg,l.0,String::from(&input_arr[2][0..input_arr[2].len()-1]));
                        }
                    }
                });
                if !x {
                    if let Err(e) = msg.reply(&ctx,"channel name not found in this guild (make sure the bot has access to the channel)"){
                        println!("error sending message {}",e);
                    }
                }



            }
            //if f.1.read().name.eq(channelsearch){
        });
        if !a{
            if let Err(e) = msg.reply(ctx,"server not found, please make sure the bot is in the server and has perms"){
                println!("Error occurred:{}",e);
            }
        }
    }else{
        if let Err(e) = msg.reply(ctx,"please read the example in ;help (dont forget the curly braces)"){
            println!("Error occurred:{}",e);
        }
    }
    Ok(())
}
#[command]
#[description = "Get a copy of the spreadsheet .csv export"]
fn export(ctx: &mut Context, msg: &Message) -> CommandResult{
    if let Err(why) = msg.author.direct_message(ctx,|ret|{
        ret.add_file("export.csv");
        ret.embed(|r|
            r.description("Spreadsheet export: ")
        );
        ret
    }){
        println!("Error sending message: {:?}",why);
    };
    Ok(())
}

#[command]
#[description = "Get a link for the bot to spread more joys of spreadsheets"]
fn invite(ctx: &mut Context, msg: &Message) -> CommandResult {
    let user = match ctx.http.get_current_user() {
        Ok(user) => user,
        Err(_) => {
            let _ = msg.reply(ctx, "An error occurred.");
            return Ok(());
        }
    };
    let link = format!("Share the spreadsheet chaos with this link:\nhttps://discordapp.com/api/oauth2/authorize?client_id={}&permissions=0&scope=bot", user.id);
    if let Err(why) = msg.author.direct_message(ctx,|ret|{
        ret.embed(|r|
            r.description(&link).color((0,255,0))
        );
        ret
    }){
        println!("Error sending message: {:?}",why);
    };
    Ok(())
}
#[command]
#[description = "get the current ping of the bot"]
fn ping(ctx: &mut Context, msg: &Message)-> CommandResult{
    let data = &ctx.data.read();
    let shard_manager = match data.get::<ShardManagerContainer>(){
        Some(t) => t,
        None =>{
            let _ = msg.reply(&ctx, "there was a problem with the shard manager");
            return Ok(());
        }
    };
    let manager = shard_manager.lock();
    let runners = manager.runners.lock();

    let runner = match runners.get(&ShardId(ctx.shard_id)){
        Some(runner)=> runner,
        None=>{
            let _ = msg.reply(&ctx,"No Shard Found");
            return Ok(());
        }
    };
    let rtr = String::from(format!("> The shard latency is `{}`",runner.latency.unwrap().as_secs()));

    if let Err(why) = msg.channel_id.say(ctx.clone(),rtr){
        println!("error happened {}",why);
    }
    Ok(())

}
#[command]
#[owners_only]
fn interroles(ctx: &mut Context, msg: &Message)-> CommandResult{
    //TODO
    Ok(())
}
#[command]
#[owners_only]
#[description = "bot conifg, type 'true' for interserver roles (when released) "]
fn config(ctx: &mut Context, msg: &Message)-> CommandResult{
    if let Err(e) = msg.author.direct_message(ctx,|m|{
        m.embed(|e|{
            e.description("Inter-server roles feature not ready yet");
            e
        });
       m
    }){
        println!("Error sending message {}",e);
    }
    Ok(())
}
fn embed_sender(ctx: &mut Context, msg:&Message, channel: &ChannelId,content: String){
    if let Err(e) = channel.send_message(&ctx.http,|r|{
        r.embed(|e|{
            e.description(&content);
            e.color((0,230,0));
            e.footer(|f|{
                f.text(format!("requested by {}#{} in server: {}",msg.author.name,msg.author.tag(),&msg.guild(&ctx).as_ref().unwrap().read().name));
                f.icon_url(msg.author.avatar_url().expect("https://discordapp.com/assets/dd4dbc0016779df1378e7812eabaa04d.png"));
                f
            });
            e
        });
        r
    }){
        println!("Error sending message {}",e);
    }

}
#[command]
#[description="this command determines a random number in a range (ex ;r 6)"]
#[aliases("r")]
fn roll(ctx: &mut Context, msg: &Message)-> CommandResult{

    let input = &msg.content;
    let input_arr:Vec<String> = input.splitn(2," ").map(|x| x.to_string()).collect();
    if input_arr.len() >=2{
        let mut rng = rand::thread_rng();
        let num: u8 = match input_arr[1].trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>(){
            Ok(e)=> e,
            Err(_r)=> 6,
        };
        let roll = rng.gen_range(0,num);
        embed_sender(ctx,msg,&msg.channel_id,format!("bot rolled: {}",roll));
    }else{
        if let Err(e) = msg.reply(&ctx,"Please specify a real whole number"){
            println!("error sending message {}",e);
        };
    }
    Ok(())
}
#[command]
#[description="get current time in UTC"]
#[aliases("time")]
fn curtime(ctx: &mut Context, msg: &Message)-> CommandResult{
    let dt = Utc::now();
    embed_sender(ctx,msg,&msg.channel_id,String::from(dt.format("%a %b %e %T %Y").to_string()));
    Ok(())
}
#[command]
#[description = "telephone to another channel on the server with ';t{channel name}{msg contents}' (dont forget to the leave the curly braces))"]
#[aliases("t")]
#[only_in(guilds)]
fn telephone(ctx: &mut Context, msg: &Message)-> CommandResult{
    if !msg.is_private() {
        let string = ctx.clone();
        let input: String = {
            if msg.content.contains(";t "){
                String::from(format!("{}",msg.content.replace(";t ", "")))
            }else{
                String::from(format!("{}",msg.content.replace(";telephone ", "")))
            }
        };
        let mut input_arr:Vec<String> = input.splitn(2,"}{").map(|x| x.to_string()).collect();

        if input_arr.len() >=2 {
            let mut x:bool = false;
            let guildlock =  &msg.guild(&ctx);
            let test =&guildlock.as_ref().unwrap().read().channels;
            if &input_arr[0].len() -1 == 0{
                if let Err(e) = msg.reply(&ctx,"Please specify a real channel name"){
                    println!("error sending message {}",e);
                };
            }
            let channelsearch = &input_arr[0][1..input_arr[0].len()];
            test.into_iter().for_each(|f|{
                if f.1.read().name.eq(channelsearch){
                    if &input_arr[1].len() -1 == 0{
                        x = true;
                        embed_sender(ctx,msg,f.0, " ".to_string());
                }else{
                        x= true;
                        embed_sender(ctx,msg,f.0,String::from(&input_arr[1][0..input_arr[1].len()]));
                    }
                }
            });
            if !x {
                if let Err(e) = msg.reply(&ctx,"channel name not found in this guild (make sure the bot has access to the channel)"){
                    println!("error sending message {}",e);
                }
            }
        }else{
            if let Err(e) = msg.reply(&ctx,"Please specify channel"){
                println!("error sending message {}",e);
            }
        }
    }
    Ok(())

}
#[command]
#[description = "A helpful command that formulates instructions for operations of the spreadsheet"]
#[aliases("sh")]
fn spreadsheethelp(ctx: &mut Context, msg: &Message)-> CommandResult{
                let url = "https://discordapp.com/api/oauth2/authorize?client_id=684150439721304095&permissions=0&scope=bot";
                let help = format!("
                 >>> -Every command for spreadsheet-bot  starts with the prefix `;s` followed first by a space then a cell to reference on the sheet\n\
                 -A reference to a cell is done by the column letter followed by row number (ex: `a1`)\n\
                 -A cell can be set by a cell reference followed by a equal sign ( separated by a space ) (ex: `a1  = 2`)\n\
                 -A cell can be set to a string, instead of a number, when quotes are in place ( ex: `a1 = \"hello world\" `)\n\
                 -A cell could also reference other cells by putting a cell reference in the deceleration (ex: `a1 = ( b1 * 2 )` )\n\
                 they can also reference multiple cells\n\n\
                 -Spreadsheet can be printed with `;s spread`, `;s spreadsheet` ,or `;s print`\n\
                 -Spreadsheet can be cleared with the `;s clear` command, or combined with a cell ref to clear a cell (ex: `;s clear a1`)\n\
                 -Spreadsheet can be saved with the command `;s save` and loaded with `;s load` for those who want to save and load data\n\
                 -The spreadsheet can also be exported by first using the command `;s export` and then using the `;export` command\n\n\
                 The spreadsheet is the same for every server that it is on and can be changed by anyone\n\
                 (though this will change in a future update)\n\
                 Creator: ***Chilla#4568***\n\
                 (feel free to dm me with any questions, comments or concerns)\n\
                  invite the bot with this link: {}",url);
                if let Err(why) = msg.author.direct_message(ctx,|ret|{
        ret.embed(|r|
            r.title("Spreadsheet-bot command basics:").description(&help).color((0,255,0))
        );
        ret
    }){
        println!("Error sending message: {:?}",why);
    };
    Ok(())
}
#[command]
#[description = "Information about lord Spreadsheetbot"]
fn about(ctx: &mut Context, msg: &Message)-> CommandResult{
    let msg = msg.channel_id.send_message(&ctx.http,|m|{
        m.embed(|e|{
            e.title("Spreadsheet bot");
            e.image("attachment://spreadsheet_bot.png");
            e.description("Spreadsheet bot creator: Chilla#4568\nDiscord bot API Creadit: Serenity Team");
            e.footer(|f|{
                f.text("Use command `;sh` for spreadsheet help");
                f
            });
            e.color((0,255,0));
            e
        });
        m.add_file(AttachmentType::Path(Path::new("./spreadsheet_bot.png")));
        m
    });
    if let Err(why) = msg {
        println!("Error sending message {:?}",why);
    }
    Ok(())
}
#[command]
#[description = "interact with the spreadsheet"]
#[example =";s a1 = 21"]
#[aliases("s")]
fn spread(ctx: &mut Context, msg: &Message)-> CommandResult{
    let input: String = {
       if msg.content.contains(";s "){
           String::from(format!("{}",msg.content.replace(";s ", "")))
       }else{
           String::from(format!("{}",msg.content.replace(";spreadsheet ", "")))
       }
    };
    let mut l = spreadsheet::enter_command(input.parse().unwrap());
    println!("username:{},command:{}",msg.author.name,msg.content);
    println!("user id:{}, username:{}, spreadsheet \n{}",msg.author.id,msg.author.name,l);
    l = format!("\n```{}```",l);
    let data = &msg.author.name;
    let mut db = USERS.lock().map_err(|_|Error::from_raw_os_error(2))?;//May cause error

    let user_url = db[1].clone();
    let reply = msg.channel_id.send_message(&ctx.http, |m|{
        m.content(&l);
        m.embed(|e|{
            e.footer(|f|{
                f.icon_url(user_url.clone());
                f.text(format!("Last command by:{} at: {} ",db[0],db[2]));
               f
            });
            e
        });
        db[0] = data.to_string();
        db[1] = match msg.author.avatar_url().as_ref(){
            Some(e) => e.to_string(),
            None => "https://discordapp.com/assets/dd4dbc0016779df1378e7812eabaa04d.png".to_string()
        };
        db[2] = Utc::now().to_string();
        m
    });

    if let Err(e) = reply{
        println!("error sending message {}",e);
    }
    Ok(())
}
#[help]
fn spreadsheetbot_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

