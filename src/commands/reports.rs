use serenity::framework::standard::{macros::command, Args, CommandResult, ArgError};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use std::fs::File;
use vim_golf_bot::challenge::Challenge;

#[command]
#[description = "Lists the open challenges."]
#[usage = ""]
#[num_args(0)]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let mut answer = MessageBuilder::new();
    answer.push_line("The available challenges are :");

    for pin in msg.channel_id.pins(ctx).await? {
        msg.channel_id.unpin(ctx, pin).await?;
    }

    for file in Challenge::all() {
        match file {
            Ok(path) => {
                if let Ok(file) = File::open(path) {
                    if let Ok(chall) = ron::de::from_reader::<_, Challenge>(file) {
                        answer
                            .push("* ")
                            .push_mono(chall.id)
                            .push(" : ")
                            .push_line(chall.title);
                    }
                }
            }
            _ => {}
        }
    }

    let out = msg.channel_id.say(ctx, answer).await?;
    out.pin(ctx).await?;

    Ok(())
}

#[command]
#[description = "Describes the provided challenge."]
#[usage = "[challenge id]"]
#[min_args(0)]
#[max_args(1)]
async fn describe(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let chall = if args.len() >= 1 {
        args.single::<Challenge>()
    } else {
        Challenge::last().ok_or(ArgError::from(String::from("No challenge to open.")))
    };

    if let Ok(chall) = chall {
        let mut msg_builder = MessageBuilder::new();

        msg_builder
            .push("The ")
            .push_mono(chall.id)
            .push_line(" challenge is :");

        msg_builder
            .push_line("")
            .push_underline_line(chall.title)
            .push_line("");

        msg_builder
            .push_line(chall.description)
            .push_line("");

        msg_builder.push_bold_line("Input:");

        msg_builder
            .push_line(chall.input.as_markdown())
            .push_line("");

        msg_builder.push_bold_line("Output :");
        msg_builder
            .push_line(chall.output.as_markdown())
            .push_line("");

        msg.channel_id.say(ctx, msg_builder.build()).await?;
    } else {
        msg.reply(ctx, "Impossible to open this challenge.").await?;
    }

    Ok(())
}

#[command]
#[aliases("leaderboard")]
#[description = "Prints the submissions for the provided challenges."]
#[usage = "[challenge id]"]
#[min_args(0)]
#[max_args(1)]
async fn submissions(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    fn describe(chall: &mut Challenge, builder: &mut MessageBuilder) {
        chall.scores.sort_by_key(|c| c.score);

        if !chall.scores.is_empty() {
            builder
                .push("Submissions for ")
                .push_mono(&chall.id)
                .push_line(" :");

            for (index, sub) in chall.scores.iter().enumerate() {
                builder
                    .push(format!("{}. ", index + 1))
                    .push_bold(&sub.author)
                    .push(" with : ")
                    .push_mono(&sub.keys)
                    .push_line(format!(" ({} pts).", sub.score));
            }
        } else {
            builder
                .push("No submissions for ")
                .push_mono(&chall.id)
                .push_line(".");
        }
    }

    let mut builder = MessageBuilder::new();

    if args.is_empty() {
        if let Some(mut chall) = Challenge::last() {
            describe(&mut chall, &mut builder);
        }
    } else {
        for mut chall in args.iter::<Challenge>().filter_map(|c| c.ok()) {
            describe(&mut chall, &mut builder);
            builder.push_line("");
        }
    }

    msg.channel_id.say(ctx, builder.build()).await?;

    Ok(())
}
