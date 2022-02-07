use std::env;

use serenity::model::guild::PartialGuild;
use serenity::model::id::{ChannelId, UserId};
use serenity::model::prelude::{GuildId, Interaction, InteractionResponseType};

use serenity::{
    async_trait,
    model::{channel::{Message}, gateway::Ready},
    prelude::*,
};

// use chrono::prelude::*;
use serenity::utils::MessageBuilder;

const OWNER_ID: UserId = UserId(134509976956829697); // @typecasto#0517
                                                     // const OWNER_ID: UserId = UserId(323262113873264660); // ICoE
                                                     // const KICK_CHANNEL: ChannelId = ChannelId(888501834312986635); // yuzu piracy
const KICK_CHANNEL: ChannelId = ChannelId(889360426998046780); // ok.testing
const LOG_CHANNEL: ChannelId = ChannelId(923761835583352973);


// const KICK_TEXT: &'static str =
//     "*Now you've done it...*\n\n\
//     You posted in #post-here-to-get-kicked, and got kicked.\n\
//     The reason for this channel's existence is to combat spambots that post the same message in every channel.\n\
//     If you didn't intentionally post in that chat, :warning: **CHANGE YOUR PASSWORD NOW.** :warning:\n\
//     If you did intentionally post in that chat (why?), skip to step 3 below.\n\
//     You've probably been hacked, and the hacker posted sketchy links in every channel they had access to.\n\
//     You may want to go through your recent DMs and delete any sketchy links coming from your account.\n\n\
//     To regain access to the server:\n\
//     1. Enable 2FA on your account. (optional, but highly recommended)\n\
//     2. Change your password.\n\
//     3. Click the blue button below that says \"Send me an invite\" (any other invites won't work, as you're currently banned)\n\
//     4. Join the server\n\
//     5. If you care about your XP, find the last time you ranked up (search is nice for this) and send a link to it in #suggestions with a message about how you want your xp back.";

// const LOG_TEXT: &'static str = "--- Spambot Kicked ---\n\
//     Username: `{}#{}`\n\
//     ID: `{}`\n\
//     Date: `<t:{}:f>`\n\
//     Original message text: \n{}";

async fn generate_kick_private_message(message: &Message, ctx: &Context) -> String {
    let guild_name = &message.guild(&ctx.cache).await
        .and_then(|g| Some(g.name))
        .or( Some(String::from("an unknown guild")) )
        .unwrap(); // Either there is a string here or I've done something terribly wrong
    MessageBuilder::new()
        .push_line(format!("You've been kicked from {} for being a suspected spambot.", guild_name) )
        .push_line("Feel free to rejoin once you've secured your account.")
        // .push_line("You may want to check your recent DMs, to see if your account has sent any sketchy links.")
        .push_line("Change your password, enable 2FA, and don't click on any sketchy links from now on.")
        .build()
}

fn generate_kick_log_message(message: &Message, could_pm: bool) -> String {
    /*
    --- Spambot Kicked ---
    Username: `User#1234`
    ID: `10203040506`
    Sent a PM: No
    Date: <t:1641780709:f>
     */
    MessageBuilder::new()
        .push_line("--- Spambot Kicked ---")
        .push("Username: ")
        // .push_mono_line(format!("{}#{:0>4}", &message.author.name, &message.author.discriminator))
        .push_mono_line(&message.author.tag())
        .push("ID: ")
        .push_mono_line(&message.author.id)
        .push_line(format!("Sent a PM: {}", if could_pm { "Yes" } else { "No" }))
        .push_line(format!("Avatar: {}", &message.author.face()))
        // .push_line(format!("Date: <t:{}:f>", &Utc::now().timestamp()))
        // .push_line("Original message:")
        // .push_safe(&message.content.replace("://", " : "))
        .build()
}

struct Handler;


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        // Stage 1: Detect
        // Return if the message is in a different channel, or by a member of a protected role
        if new_message.channel_id != KICK_CHANNEL || new_message.author.id == OWNER_ID {
            // FEATURE: check for list of roles, rather than hardcoded ID
            return;
        }

        // Stage 2: Warn
        // Send them a private message
        // new_message.author.create_dm_channel(&ctx.http).await
        //     .and_then(|c| c.send_message(&ctx.http, |create_message| async move {
        //         create_message.content(generate_kick_private_message(&new_message, &ctx).await)
        //     }.await
        //     ));
        let could_private_message;
        let private_message_text = generate_kick_private_message(&new_message, &ctx).await;
        if let Ok(dm_channel) = new_message.author.create_dm_channel(&ctx.http).await {
            could_private_message = dm_channel
                .send_message(&ctx.http, |m| m.content(private_message_text)).await
                .is_ok();
        } else {
            could_private_message = false;
        }

        // Stage 3: Ban
        // Ban them, deleting 1 day of messages and kicking them, then unban them.
        if let Some(guild) = &new_message.guild(&ctx.cache).await {
            if let Err(_) = guild.ban_with_reason(&ctx.http, &new_message.author.id, 1, "Spambot (autobanned)").await {
                return; // can't ban this user, return.
            }
            let _ = guild.unban(&ctx.http, &new_message.author.id).await;
        } else {eprintln!("Failed to find guild.")}

        // Stage 4: Log
        // Send a log message, simple.
        if let Some(log_channel) = &ctx.cache.guild_channel(LOG_CHANNEL).await {
            log_channel.say(&ctx.http, generate_kick_log_message(&new_message, could_private_message)).await
                .expect("Failed to send log message.");
        }
        else {eprintln!("Failed to find log channel.")}
    }

    // Fired when bot is ready
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected and ready to go.", ready.user.name)
    }

    // async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    //     dbg!(&interaction);
    //     match interaction {
    //         Interaction::Ping(_) => {}
    //         Interaction::MessageComponent(component_interaction) => {
    //             match &component_interaction.data.custom_id {
    //                 unban_string if unban_string.starts_with("unban_from: ") => {
    //                     // println!("We're unbanning with string {}", unban_string);

    //                     let id = GuildId(
    //                         unban_string
    //                             .strip_prefix("unban_from: ")
    //                             .unwrap()
    //                             .parse::<u64>()
    //                             .expect("failed to parse guildid"),
    //                     );

    //                     let pg: PartialGuild = PartialGuild::get(&ctx.http, id).await.unwrap();

    //                     match pg.bans(&ctx.http).await {
    //                         Ok(bans) => {
    //                             for ban in bans {
    //                                 if ban.user.id == component_interaction.user.id {
    //                                     if ban.reason.unwrap_or("".to_string()).contains("Spambot")
    //                                     {
    //                                         &pg.unban(&ctx.http, &ban.user.id);
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                         Err(why) => {
    //                             eprintln!("ERROR: {}", why)
    //                         } // user was unbanned already, and guild has no other bans?
    //                     }

    //                     // done with response, send back to server
    //                     &component_interaction
    //                         .create_interaction_response(&ctx.http, |response| {
    //                             response
    //                                 .kind(InteractionResponseType::UpdateMessage)
    //                                 .interaction_response_data(|x| x)
    //                         })
    //                         .await;
    //                 }
    //                 _ => {
    //                     eprintln!("ERROR: Reached _ pattern in MessageComponent.")
    //                 }
    //             }
    //         }
    //         Interaction::ApplicationCommand(_command_interaction) => {
    //             // do something here later
    //         }
    //     }
    // }
}

#[tokio::main]
async fn main() {
    // Token from environment
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a discord token from environment variable $DISCORD_TOKEN.");

    // Make bot
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .application_id(888489827492827206)
        .await
        .expect("bot create error.");

    // Start bot
    if let Err(error) = client.start().await {
        eprintln!("Client error: {:?}", error);
    }
}