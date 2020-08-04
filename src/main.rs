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

use commands::{manage::*, participate::*, reports::*};

struct Handler;

impl EventHandler for Handler {}

#[group]
#[commands(register, list, describe, participate, close, submissions)]
struct General;

// The framework provides two built-in help commands for you to use.
// But you can also make your own customized help command that forwards
// to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip = "Hi !\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
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
    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    env_logger::init();

    let token = env::args()
        .next_back()
        .expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
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
