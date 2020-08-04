use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use sha1::{Digest, Sha1};

use std::fs::File;
use std::str::Lines;
use vim_golf_bot::challenge::Challenge;

fn extract_content(content: &mut Lines) -> (String, Vec<String>, Vec<String>) {
    let mut input_lines: Vec<String> = Vec::new();
    let mut output_lines: Vec<String> = Vec::new();

    let mut filling = &mut input_lines;
    let mut is_filling = false;

    content.next();

    let mut content = content.skip_while(|line| line.is_empty());

    let line = content.next().unwrap_or("No title");

    let first: String;
    if let Some(end) = line.strip_prefix("# ") {
        first = String::from(end);
    } else {
        is_filling = line == "```";
        first = String::from("No title");
    }

    for line in content {
        if line == "```" {
            if is_filling {
                // Finished reading the first block
                is_filling = false;
                filling = &mut output_lines;
            } else {
                // Starting to read either of the two blocks
                is_filling = true;
            }
        } else if is_filling {
            filling.push(line.to_owned());
        }
    }

    (first, input_lines, output_lines)
}

#[command]
async fn register(ctx: &Context, msg: &Message) -> CommandResult {
    let (title, input_lines, output_lines) = extract_content(&mut msg.content.lines());

    if input_lines.is_empty() || output_lines.is_empty() {
        msg.reply(
            ctx,
            "Invalid vim golf challenge : missing challenge content.",
        )
        .await?;
        return Ok(());
    }

    // Create unique challenge name
    let mut hasher = Sha1::new();

    hasher.update(msg.author.name.as_bytes());
    hasher.update(msg.timestamp.to_string().as_bytes());
    hasher.update(title.as_bytes());

    let mut chal_id = String::with_capacity(6);

    for elem in hasher.finalize().iter().take(3) {
        chal_id.push_str(&format!("{:02x}", elem));
    }

    let chall = Challenge::new(title, input_lines, output_lines, chal_id);

    let file = File::create(Challenge::filename(&chall.id))?;
    ron::ser::to_writer(file, &chall)?;

    msg.reply(
        ctx,
        format!(
            "Thanks for your submission, your challenge id is `{}`",
            chall.id
        ),
    )
    .await?;

    Ok(())
}

#[command]
async fn close(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(mut chall) = args.single::<Challenge>() {
        std::fs::remove_file(Challenge::filename(&chall.id))?;

        chall.scores.sort_by(|a, b| a.score.cmp(&b.score));

        let mut builder = MessageBuilder::new();

        builder
            .push("Succesfully closed")
            .push_mono(chall.id)
            .push_line("");

        for winner in chall.scores.iter().take(5) {
            builder
                .push("* ")
                .push_bold(&winner.author)
                .push(" with ")
                .push_mono(&winner.keys)
                .push_line(format!(" ({} keys)", winner.score));
        }

        msg.channel_id.say(ctx, builder.build()).await?;
    } else {
        msg.reply(ctx, "Invalid command: invalid challenge id.")
            .await?;
    }

    Ok(())
}
