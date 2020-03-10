//use "*" to configure bot
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};
#[command]
pub fn servers(ctx:&mut Context, msg: &Message, mut args: Args)-> CommandResult{
    Ok(())
}