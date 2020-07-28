extern crate chrono;
use crate::{models, Bans, DbPool, USERS, commands::spreadsheet};
use rand::Rng;
use chrono::{Datelike, Timelike, Utc};
use serenity::{
    framework::standard::ArgError::Parse,
    http::{AttachmentType, routing::RouteInfo::CreateMessage},
    prelude::*,
    model::{prelude::{UserId,Permissions},channel::{Message,Embed},
        id::ChannelId, channel::Channel,
    },
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
          sync::Arc,path::Path,io::Error
};
use serenity::http::routing::Route::GuildsId;
use serenity::model::guild::Guild;
use serenity::model::id::GuildId;

pub(crate) struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer{
    type Value = Arc<Mutex<ShardManager>>;
}

#[command]
#[owners_only]
#[description ="get a list of servers that spreadsheet bot is in"]
fn servers(ctx: &mut Context,msg:&Message)->CommandResult{
    let a = &ctx.cache.read().guilds;
    &ctx.cache.read().guilds.iter().for_each(|(guildid,guild)|{
       println!("server name`{}`",guild.read().name);
        println!("roles: ");
        guild.read().roles.iter().for_each(|(x,y)|{
            print!("{}, ",y.name);

        });
        println!("channels: ");
        guild.read().channels.iter().for_each(|(x,y)|{
           print!("{}, ",y.read().name);
        });
    });
    Ok(())
}
#[command]
#[description = "inter server message into another servers channel. Usage ;tl {server_name}{channel_name}{message_content}"]
#[aliases("tl")]
#[only_in(guilds)]
fn telelink(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult{
    let string = ctx.clone();
    let server_name = args.single::<String>();
    let channel_id = args.single::<ChannelId>();
    let data = args.rest();

    let server = match server_name{
        Ok(content)=>{
            content.to_string()
        }
        Err(_)=>{
            let _ = msg.reply(&ctx, &format!("please specify a guild name "));
            return Ok(());
        }
    };

    let channel = match channel_id{
        Ok(channel_id)=>{
            let channel = channel_id.to_channel(&ctx).unwrap();
            channel
        }
        Err(Parse(e))=>{
            let _ = msg.reply(&ctx, &format!("please specify a valid channel ({})",e));
            return Ok(());
        }
        Err(_e)=>{
            let _ = msg.reply(&ctx,"please specify a valid channel");
            return Ok(())
        }
    };
    &ctx.cache.read().guilds.iter().for_each(|(guild,guildlock)|{
        if guildlock.read().name == server{
            embed_sender(&mut ctx.clone(), msg, &channel.id(), data.parse().unwrap())
        }
    });
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
#[description = "Spreadsheet bot will print the spreadsheet in a different channel(s) with the spreadsheet command(ex ;ss #general #channel1 #channel2)"]
#[aliases("ss")]
#[only_in(guilds)]
fn sendspread(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    for _ in 0..args.len() {
        let channel_id = args.single::<ChannelId>();
        let channel = match channel_id {
            Ok(channel_id) => {
                let channel = channel_id.to_channel(&ctx).unwrap();
                channel
            }
            Err(Parse(e)) => {
                let _ = msg.reply(&ctx, &format!("please specify a valid channel ({})", e));
                return Ok(())
            }
            Err(_e) => {
                let _ = msg.reply(&ctx, "please specify a valid channel");
                return Ok(())
            }
        };
        if let Err(e) = &channel.id().send_message(ctx.clone(), |content|
            content.content(format!("```{}```", spreadsheet::get_spread()))
        ) {
            println!("error sending message {}", e);
        };
    }
    if let Err(e) = msg.reply(&ctx, "Message sent") {
        println!("error sending message {}", e);
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
                f.text(format!("requested by {} in server: {}",msg.author.tag(),&msg.guild(&ctx).as_ref().unwrap().read().name));
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
#[description="this command determines a random number in a range (ex ;r 6) defaults to 6"]
#[aliases("r")]
fn roll(ctx: &mut Context, msg: &Message, mut args: Args)-> CommandResult{
    let contents = match args.single::<String>(){
        Ok(content)=>{
            let content = content.to_string();
            content
        }
        Err(_)=>{
            "6".to_string()
        }
    };
        let mut rng = rand::thread_rng();
        let num: u8 = match contents.trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u8>(){
            Ok(e)=> e,
            Err(_r)=> 6,
        };
        let roll = rng.gen_range(0,num);
        embed_sender(&mut ctx.clone(),msg,&msg.channel_id,format!("bot rolled: {}",roll));
    Ok(())
    }
#[command]
#[description="get current time in UTC"]
#[aliases("time")]
fn curtime(ctx: &mut Context, msg: &Message)-> CommandResult{
    let dt = Utc::now();
    embed_sender(&mut ctx.clone(),msg,&msg.channel_id,String::from(dt.format("%a %b %e %T %Y").to_string()));
    Ok(())
}
#[command]
#[description = "telephone to another channel on the server(s) with ';t #channel \"msg contents\" ' \n this command could also be used on other server channels"]
#[aliases("t")]
#[only_in(guilds)]
fn telephone(ctx: &mut Context, msg: &Message, mut args: Args)-> CommandResult{
    let channel_id = args.quoted().single::<ChannelId>();
    let msg_contents = args.quoted().single::<String>();

    let channel = match channel_id{
        Ok(channel_id)=>{
            let channel = channel_id.to_channel(&ctx).unwrap();
            channel
        }
        Err(Parse(e))=>{
            let _ = msg.reply(&ctx, &format!("please specify a valid channel ({})",e));
            return Ok(());
        }
        Err(_e)=>{
            let _ = msg.reply(&ctx,"please specify a valid channel");
            return Ok(())
        }
    };
    let contents = match msg_contents{
        Ok(content)=>{
            let content = content.to_string();
            content
        }
        Err(_)=>{
            " ".to_string()
        }
    };
    embed_sender(&mut ctx.clone(), msg, &channel.id(), contents);
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
                 -Averaging a range can be done with the 'AVG' command within a formula cell (ex: `b5 = ( AVG b1-b4)`)\n\
                 -Computing the sum of a range can be done with the 'SUM' command within a formula cell (ex: `b5 = SUM b1-b4`)\n\
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
fn spread(ctx: &mut Context, msg: &Message, mut args: Args)-> CommandResult{
    let input = args.rest();

    let mut l = spreadsheet::enter_command(input.parse()?);
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

