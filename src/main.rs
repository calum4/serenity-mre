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
use strum::IntoEnumIterator;

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

#[derive(strum::EnumIter)]
enum SelectMenuKind {
    String,
    User,
    Role,
    Mentionable,
    Channel,
}

impl SelectMenuKind {
    fn as_str(&self) -> &'static str {
        match self {
            SelectMenuKind::String => "string",
            SelectMenuKind::User => "user",
            SelectMenuKind::Role => "role",
            SelectMenuKind::Mentionable => "mentionable",
            SelectMenuKind::Channel => "channel",
        }
    }
}

impl FromStr for SelectMenuKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(SelectMenuKind::String),
            "user" => Ok(SelectMenuKind::User),
            "role" => Ok(SelectMenuKind::Role),
            "mentionable" => Ok(SelectMenuKind::Mentionable),
            "channel" => Ok(SelectMenuKind::Channel),
            _ => Err(())
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn dispatch(&self, context: &Context, event: &FullEvent) {
        const MODAL_CUSTOM_ID: &str = "example_modal";
        const SELECT_MENU_CUSTOM_ID: &str = "example_select_menu";

        match event {
            FullEvent::Ready { .. } => {
                let mut msg = CreateMessage::new()
                    .content("## Modal Values MRE\n\n Click the button below!");

                for variant in SelectMenuKind::iter() {
                    msg = msg.button(CreateButton::new(variant.as_str()).label(variant.as_str()));
                }

                self.channel_id
                    .send_message(context.http(), msg)
                    .await
                    .unwrap();
            }
            FullEvent::InteractionCreate { interaction, .. } => match interaction {
                Interaction::Component(interaction) => {
                    if interaction.channel_id != self.channel_id {
                        return;
                    }

                    let Ok(select_menu_kind) = SelectMenuKind::from_str(interaction.data.custom_id.as_str()) else {
                        return
                    };

                    let string_select_menu_options = [
                        CreateSelectMenuOption::new("Option 1", "option-1"),
                        CreateSelectMenuOption::new("Option 2", "option-2"),
                    ];

                    let select_menu = match select_menu_kind {
                        SelectMenuKind::String => {
                            CreateSelectMenu::new(
                                SELECT_MENU_CUSTOM_ID,
                                CreateSelectMenuKind::String {
                                    options: Cow::Borrowed(string_select_menu_options.as_slice()),
                                },
                            )
                        }
                        SelectMenuKind::User => CreateSelectMenu::new(
                            SELECT_MENU_CUSTOM_ID,
                            CreateSelectMenuKind::User {
                                default_users: None,
                            },
                        ),
                        SelectMenuKind::Role => CreateSelectMenu::new(SELECT_MENU_CUSTOM_ID, CreateSelectMenuKind::Role { default_roles: None }),
                        SelectMenuKind::Mentionable => CreateSelectMenu::new(SELECT_MENU_CUSTOM_ID, CreateSelectMenuKind::Mentionable { default_users: None, default_roles: None }),
                        SelectMenuKind::Channel => CreateSelectMenu::new(SELECT_MENU_CUSTOM_ID, CreateSelectMenuKind::Channel { channel_types: None, default_channels: None }),
                    };

                    let modal_components = [CreateComponent::Label(CreateLabel::select_menu(
                        "Select Menu",
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
