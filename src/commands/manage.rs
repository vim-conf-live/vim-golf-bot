use serenity::framework::standard::{macros::command, Args, CommandResult, CommandError};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use log::info;
use sha1::{Digest, Sha1};

use std::fs::File;
use std::str::Lines;
use vim_golf_bot::challenge::{Challenge, FromLines, TextBlock};

fn extract_content(lines: &mut Lines) -> Result<(String, String, TextBlock, TextBlock), String> {
    lines.next();

    let mut content = lines.filter(|line| !line.is_empty());

    let line = content.next().ok_or(String::from("Challenge is empty"))?;

    let first: String;
    if let Some(end) = line.strip_prefix("# ") {
        first = String::from(end);
    } else {
        return Err(String::from("Missing title"));
    }

    let desc = String::from_lines(lines).map_err(|_| String::from("Missing description"))?;

    let input = TextBlock::from_lines(lines).map_err(|_| String::from("Missing input"))?;
    let output = TextBlock::from_lines(lines).map_err(|_| String::from("Mising output"))?;

    Ok((first, desc, input, output))
}

#[command]
#[description = r##"Registers a new challenge.

The argument for this function is actually a text, describing the challenge.

The format should be :

```
register

# Challenge Title

## Short Description

Input:
[MARKDOWN CODE BLOCK CONTAINING THE INPUT]

Output:
[MARKDOWN CODE BLOCK CONTAINING THE INPUT]
```

The code block should by separated in triple backticks (as any markdown code block).
"##]
async fn register(ctx: &Context, msg: &Message) -> CommandResult {
    match extract_content(&mut msg.content.lines()) {
        Ok((title, desc, input_lines, output_lines)) => {
            // Create unique challenge name
            let mut hasher = Sha1::new();

            hasher.update(msg.author.name.as_bytes());
            hasher.update(msg.timestamp.to_string().as_bytes());
            hasher.update(title.as_bytes());

            let mut chal_id = String::with_capacity(6);

            for elem in hasher.finalize().iter().take(3) {
                chal_id.push_str(&format!("{:02x}", elem));
            }

            let chall = Challenge::new(
                title,
                desc,
                input_lines,
                output_lines,
                chal_id,
                msg.timestamp.timestamp(),
            );

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
        Err(err) => {
            msg.reply(ctx, format!("Invalid vim golf challenge : {}.", err))
                .await?;
            Err::<(), CommandError>(CommandError::from(err))
        }
    }
}

#[command]
#[allowed_roles("Conference Admin", "VimGolf mod")]
#[description = "Closes the provided challenge."]
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
