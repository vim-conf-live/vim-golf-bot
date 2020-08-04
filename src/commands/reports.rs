use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use glob::glob;

use std::fs::File;
use vim_golf_bot::challenge::Challenge;

#[command]
#[description = "Lists the open challenges."]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let mut answer = MessageBuilder::new();
    answer.push_line("The available challenges are :");

    for file in glob("challenges/*.chal")? {
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
    if let Ok(chall) = args.single::<Challenge>() {
        let mut msg_builder = MessageBuilder::new();

        msg_builder
            .push("The ")
            .push_mono(chall.id)
            .push_line(" challenge is :");

        msg_builder
            .push_line("")
            .push_line(chall.title)
            .push_line("");

        msg_builder.push_bold_line("Input:");

        msg_builder
            .push_line("```")
            .push_line(chall.input.join("\n"))
            .push_line("```")
            .push_line("");

        msg_builder.push_bold_line("Output :");
        msg_builder
            .push_line("```")
            .push_line(chall.output.join("\n"))
            .push_line("```")
            .push_line("");

        msg.channel_id.say(ctx, msg_builder.build()).await?;
    } else {
        msg.reply(ctx, "Impossible to open this challenge.").await?;
    }

    Ok(())
}

#[command]
#[description = "Prints the submissions for the provided challenges."]
async fn submissions(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut builder = MessageBuilder::new();

    for chal in args.iter::<Challenge>().filter_map(|c| c.ok()) {
        if !chal.scores.is_empty() {
            builder
                .push("Submissions for ")
                .push_mono(chal.id)
                .push_line(" :");

            for sub in chal.scores {
                builder
                    .push("* ")
                    .push_bold(sub.author)
                    .push(" with : ")
                    .push_mono(sub.keys)
                    .push_line(format!(" ({} pts).", sub.score));
            }
        } else {
            builder
                .push("No submissions for ")
                .push_mono(chal.id)
                .push_line(".");
        }
        builder.push_line("");
    }

    msg.channel_id.say(ctx, builder.build()).await?;

    Ok(())
}
