use serenity::all::{Context, FullEvent, GenericChannelId, Interaction};
use serenity::builder::{
    CreateButton, CreateComponent, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateLabel, CreateMessage, CreateModal, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption,
};
use serenity::http::CacheHttp;
use serenity::prelude::{EventHandler, GatewayIntents};
use serenity::secrets::Token;
use serenity::{Client, async_trait};
use std::borrow::Cow;
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let token =
        Token::from_env("DISCORD_TOKEN").expect("Expected a `DISCORD_TOKEN` in the environment");

    let channel_id_string =
        env::var("DISCORD_CHANNEL_ID").expect("Expected `DISCORD_CHANNEL_ID` in the environment");
    let channel_id = GenericChannelId::from_str(channel_id_string.as_str()).unwrap();

    let mut client = Client::builder(token, GatewayIntents::non_privileged())
        .event_handler(Handler { channel_id })
        .await
        .unwrap();

    client.start().await.unwrap();
}

struct Handler {
    channel_id: GenericChannelId,
}

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, context: &Context, event: &FullEvent) {
        const OPEN_MODAL_BUTTON_ID: &str = "open_example_modal";
        const MODAL_CUSTOM_ID: &str = "example_modal";
        const STRING_SELECT_CUSTOM_ID: &str = "example_modal";

        match event {
            FullEvent::Ready { .. } => {
                let msg = CreateMessage::new()
                    .content("## Modal Values MRE\n\n Click the button below!")
                    .button(CreateButton::new(OPEN_MODAL_BUTTON_ID).label("Open Modal"));

                self.channel_id
                    .send_message(context.http(), msg)
                    .await
                    .unwrap();
            }
            FullEvent::InteractionCreate { interaction, .. } => match interaction {
                Interaction::Component(interaction) => {
                    if interaction.channel_id != self.channel_id
                        || interaction.data.custom_id != OPEN_MODAL_BUTTON_ID
                    {
                        return;
                    }

                    let select_menu_options = [
                        CreateSelectMenuOption::new("Option 1", "option-1"),
                        CreateSelectMenuOption::new("Option 2", "option-2"),
                    ];

                    let select_menu = CreateSelectMenu::new(
                        STRING_SELECT_CUSTOM_ID,
                        CreateSelectMenuKind::String {
                            options: Cow::Borrowed(select_menu_options.as_slice()),
                        },
                    );

                    let modal_components = [CreateComponent::Label(CreateLabel::select_menu(
                        "String Select",
                        select_menu,
                    ))];

                    let modal = CreateModal::new(MODAL_CUSTOM_ID, "Example Modal")
                        .components(Cow::Borrowed(modal_components.as_slice()));

                    interaction
                        .create_response(context.http(), CreateInteractionResponse::Modal(modal))
                        .await
                        .unwrap();
                }
                Interaction::Modal(interaction) => {
                    if interaction.channel_id != self.channel_id
                        || interaction.data.custom_id != MODAL_CUSTOM_ID
                    {
                        return;
                    }

                    let msg = CreateInteractionResponseMessage::new()
                        .content(format!(
                            "## `interaction.data`\n```\n{}\n```",
                            serde_json::to_string_pretty(&interaction.data).unwrap()
                        ))
                        .ephemeral(true);

                    interaction
                        .create_response(context.http(), CreateInteractionResponse::Message(msg))
                        .await
                        .unwrap();
                    dbg!(&interaction.data);
                }
                _ => return,
            },
            _ => return,
        }
    }
}
