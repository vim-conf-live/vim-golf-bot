use serenity::framework::standard::{macros::command, Args, CommandResult, ArgError};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use std::fs::File;
use vim_golf_bot::challenge::Challenge;

#[command]
#[description = "Lists the open challenges."]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let mut answer = MessageBuilder::new();
    answer.push_line("The available challenges are :");

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

    msg.channel_id.say(ctx, answer).await?;

    Ok(())
}

#[command]
#[description = "Describes the provided challenge."]
async fn describe(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let chall;
    if args.len() >= 2 {
        chall = args.single::<Challenge>();
    } else {
        chall = Challenge::last().ok_or(ArgError::from(String::from("No challenge to open.")));
    }

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
async fn submissions(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    fn describe(chall: &Challenge, builder: &mut MessageBuilder) {
        if !chall.scores.is_empty() {
            builder
                .push("Submissions for ")
                .push_mono(&chall.id)
                .push_line(" :");

            for sub in &chall.scores {
                builder
                    .push("* ")
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
        if let Some(chall) = Challenge::last() {
            describe(&chall, &mut builder);
        }
    } else {
        for chall in args.iter::<Challenge>().filter_map(|c| c.ok()) {
            describe(&chall, &mut builder);
            builder.push_line("");
        }
    }

    msg.channel_id.say(ctx, builder.build()).await?;

    Ok(())
}
