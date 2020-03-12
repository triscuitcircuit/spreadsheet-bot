extern crate libc;
use serenity::framework::standard::macros::{command,group,help,check};
use libc::{c_uchar,size_t,c_int,c_schar};
use serenity::framework::standard::{CommandResult,CheckResult};

#[command]
fn mem()->CommandResult{
    //TODO
    Ok(())
}