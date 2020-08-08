use serenity::framework::standard::{macros::command, ArgError, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::{prelude::*, utils::MessageBuilder};

use log::info;

use nvim_rs::{create::tokio as create, rpc::handler::Dummy as DummyHandler};

use tokio::process::Command;

use std::fs::File;
use vim_golf_bot::challenge::Challenge;

pub async fn create_nvim_instance(
) -> nvim_rs::neovim::Neovim<nvim_rs::compat::tokio::Compat<tokio::process::ChildStdin>> {
    const NVIMPATH: &str = "nvim";
    let handler = DummyHandler::new();

    let (nvim, _io_handle, _child) = create::new_child_cmd(
        Command::new(NVIMPATH)
            .args(&["-u", "NONE", "--embed", "--headless", "-Z", "--noplugin"])
            .env("NVIM_LOG_FILE", "nvimlog"),
        handler,
    )
    .await
    .unwrap();

    nvim
}

async fn emulate(
    input: &Vec<String>,
    keys: &str,
) -> Result<(Vec<String>, usize, Option<String>), CommandError> {
    let mut nvim = create_nvim_instance().await;
    let buf = nvim.create_buf(false, true).await?;
    let win = nvim.get_current_win().await?;

    win.set_buf(&buf).await?;
    let keys_parsed = nvim.replace_termcodes(keys, true, true, true).await?;

    buf.set_lines(0, -1, false, input.to_owned()).await?;

    info!(
        "Feeding : {}",
        keys_parsed.escape_default().collect::<String>()
    );
    nvim.feedkeys(&keys_parsed, "ntx", true).await?;

    let err = nvim.get_vvar("errmsg").await.unwrap();

    let out_lines = buf.get_lines(0, -1, false).await?;

    nvim.quit_no_save().await?;

    Ok((
        out_lines,
        keys_parsed.len(),
        err.as_str().map(|s| s.to_owned()),
    ))
}

#[command]
#[description = r##"Participate to a challenge.
This command should be called with two arguments : a challenge ID and keys (as in map rhs)
Both of the ID and the keys can be escaped within backticks.
When providing a `try` as the first argument, the input will not be submitted.
This can be used to check your input.
"##]
#[max_args(3)]
#[min_args(1)]
#[usage("['try'] [challenge id] {key sequence}")]
pub async fn participate(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (ref mut chall, ref keys, is_try) = match args.len() {
        1 => (
            Challenge::last().ok_or(ArgError::from(String::from("No challenge to open.")))?,
            args.single::<String>().unwrap(),
            false,
        ),
        2 => {
            // Two arguments is tricky, but not ambiguate
            if let Some("try") = args.current() {
                // [try] {seq}
                args.advance();
                (
                    Challenge::last()
                        .ok_or(ArgError::from(String::from("No challenge to open.")))?,
                    args.single::<String>()?,
                    true,
                )
            } else {
                // [challenge id] {seq}
                (args.single::<Challenge>()?, args.single::<String>()?, false)
            }
        }
        3 => (args.single::<Challenge>()?, args.single::<String>()?, true),

        _ => unreachable!(),
    };

    let keys = keys.strip_prefix('`').unwrap_or(keys);
    let keys = keys.strip_suffix('`').unwrap_or(keys);

    let (out_lines, score, err) = emulate(&chall.input.content, &keys).await?;

    if chall.output.content.eq(&out_lines) {
        msg.reply(
            ctx,
            format!("Your submission is valid ! Your score is : {}", score),
        )
        .await?;

        const DM_CHAN: &str = "DM with";

        let channel_name: String = msg
            .channel_id
            .name(ctx)
            .await
            .unwrap_or(String::from(DM_CHAN));

        if !(is_try || channel_name.starts_with(DM_CHAN)) {
            chall.add_submission(msg.author.name.to_string(), keys.to_owned(), score);
            let file = File::create(Challenge::filename(&chall.id))?;
            ron::ser::to_writer(file, &chall)?;
        }
    } else {
        let lang = if let Some(lang) = &chall.output.lang {
            lang.to_string()
        } else {
            String::new()
        };

        let mut builder = MessageBuilder::new();
        builder
            .push_underline("Invalid answer")
            .push(", your result is : ")
            .push("```")
            .push_line(lang)
            .push_line(out_lines.join("\n"))
            .push_line("```");

        if let Some(err) = err {
            if !err.is_empty() {
                builder
                    .push_line("")
                    .push_line("An error occurred when executing your input :")
                    .push_line("```")
                    .push_line(err)
                    .push_line("```");
            }
        }

        msg.reply(ctx, builder.build()).await?;
    }

    Ok(())
}
