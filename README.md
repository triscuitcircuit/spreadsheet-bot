# Spreadsheet-bot[![Discord Bots](https://top.gg/api/widget/status/684150439721304095.svg)](https://top.gg/bot/684150439721304095)  [![Build Status](https://travis-ci.com/Triscuit-circuit/spreadsheet-bot.svg?token=SfpfaTZk1PvqvrRX4uGo&branch=master)](https://travis-ci.com/Triscuit-circuit/spreadsheet-bot)   [![Discord Bots](https://top.gg/api/widget/servers/684150439721304095.svg)](https://top.gg/bot/684150439721304095) [![Discord Bots](https://top.gg/api/widget/lib/684150439721304095.svg)](https://top.gg/bot/684150439721304095)






<div style="text-align: center"><a href="https://top.gg/bot/684150439721304095" >
  <img src="https://top.gg/api/widget/684150439721304095.svg" alt="Spreadsheet-bot" />
</a></div>


## Table of Contents
* [Spreadsheet Bot](#spreadsheet-bot)
* [About the bot](#about-the-project)
* [Self Hosting](#self-hosting)
* [Features](#features)

## Spreadsheet Bot
Have you guys ever wanted a spreadsheet bot for discord where it emulates being at work?? well, now your oddly specific dreams of being a workaholic on Discord have come true.

With this bot ([Bot Invite Link](https://discordapp.com/api/oauth2/authorize?client_id=684150439721304095&permissions=0&scope=bot)) you can make it seem like you are working hard at the office. Just type **;help** to get started with your deep delve into Spreadsheet insanity.

Not only does this bot act as a spreadsheet, but it is the same spreadsheet across all the servers that it is in.

You can never get away from spreadsheets.NEVER. NEVER EVER EVER.

## About The Project
This bot is built with Rust Serenity Discord API, Rust Diesel, Serde, lazy_static, and Postgres.

Made by: [Triscuit](https://github.com/Triscuit-circuit) 


## Self Hosting
1. Clone the repo
```shell script
git clone https://github.com/Triscuit-circuit/spreadsheet-bot.git
```
2. Install the diesel_cli crate
```shell script
cargo install diesel_cli
```
3. Install Postgres 12 and setup the Database with Diesel in the migrations folder
```shell script
diesel setup
```
4. Run Diesel migrations Setup
```shell script
diesel migrations run
```
5. Build Cargo packages and run the repo. 
```shell script
DISCORD_TOKEN="token" DATABASE_URL="localhosturl" cargo run
```

## Features
Features to come soon:
- [x] command to export sheet as **.CSV** from bot
- [ ] inter-server roles function
- [ ] bot config
- [x] fix spreadsheet circular reference crash
- [ ] admin config and command library
- [ ] private spreadsheet config instead of inter-server
- [ ] admin changelog for servers config
- [ ] role assignment for inter-server
