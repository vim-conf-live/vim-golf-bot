//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```

#[macro_use]
extern crate serde;

mod commands;

use log::{error, info};
use serenity::{
    framework::{
        standard::{
            help_commands,
            macros::{group, help, hook},
            Args, CommandGroup, CommandResult, HelpOptions,
        },
        StandardFramework,
    },
    http::Http,
    model::prelude::*,
    prelude::*,
};
use std::collections::HashSet;
use std::env;

mod challenge;
use challenge::Challenge;

use commands::{manage::*, participate::*, reports::*};

struct Handler;

impl EventHandler for Handler {}

#[group]
#[commands(register, list, describe, participate, close, submissions)]
struct General;

#[help]
#[individual_command_tip = "Hi !\n\
If you want more information about a specific command, just pass the command as argument."]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[tokio::main]
async fn main() {

    // Set up the challenges/ directory if needed
    // TODO(vigoux): remove that unwrap
    Challenge::create_dir().unwrap();

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    env_logger::init();

    let token = env::args()
        .next_back()
        .expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // Let's fetch bot ID
    let bot_id = http
        .get_current_application_info()
        .await
        .map(|info| info.id)
        .ok();

    let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(
            StandardFramework::new()
                .configure(|c| c.prefix("?").on_mention(bot_id))
                .after(handle_after)
                .group(&GENERAL_GROUP)
                .unrecognised_command(unknown_command)
                .help(&MY_HELP),
        )
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

#[hook]
async fn handle_after(_ctx: &Context, _msg: &Message, command_name: &str, error: CommandResult) {
    match error {
        Ok(()) => {
            info!("Executed {} succesfully.", command_name);
        }
        Err(why) => error!("Error executing {} : {}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    info!("Unknown command : {}", unknown_command_name);
}
