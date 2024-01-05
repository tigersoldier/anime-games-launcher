use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::games;
use crate::games::integrations;

use crate::config;
use crate::config::driver::Driver;

use crate::LAUNCHER_FOLDER;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Games {
    pub integrations: Driver,
    pub settings: HashMap<String, GameSettings>
}

impl Default for Games {
    #[inline]
    fn default() -> Self {
        Self {
            integrations: Driver::PhysicalFsDriver {
                base_folder: LAUNCHER_FOLDER.join("integrations")
            },

            settings: HashMap::new()
        }
    }
}

impl From<&Json> for Games {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            integrations: value.get("integrations")
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or(default.integrations),

            settings: match value.get("settings").and_then(Json::as_object) {
                Some(values) => {
                    let mut settings = HashMap::new();

                    for (name, game) in values {
                        settings.insert(name.to_owned(), GameSettings::from(game));
                    }

                    settings
                }

                None => HashMap::new()
            }
        }
    }
}

impl Games {
    pub fn get_game_settings(&self, game: impl AsRef<str>) -> anyhow::Result<GameSettings> {
        match self.settings.get(game.as_ref()) {
            Some(settings) => Ok(settings.to_owned()),
            None => {
                let Some(game_object) = games::get(game.as_ref())? else {
                    anyhow::bail!("Couldn't find {} integration script", game.as_ref());
                };

                let settings = GameSettings::default_for_game(game_object)?;

                config::set(format!("games.settings.{}", game.as_ref()), serde_json::to_value(settings.clone())?)?;

                Ok(settings)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSettings {
    pub paths: HashMap<String, GameEditionPaths>
}

impl GameSettings {
    pub fn default_for_game(game: &integrations::Game) -> anyhow::Result<Self> {
        let editions = game.get_game_editions_list()?;

        Ok(Self {
            paths: editions.into_iter().filter_map(|edition| {
                match GameEditionPaths::default_for_edition(game, &edition) {
                    Ok(paths) => Some((edition.name, paths)),
                    Err(_) => None
                }
            }).collect()
        })
    }
}

impl From<&Json> for GameSettings {
    #[inline]
    fn from(value: &Json) -> Self {
        Self {
            paths: match value.get("paths").and_then(Json::as_object) {
                Some(values) => {
                    let mut paths = HashMap::new();

                    for (name, path) in values.clone() {
                        if let Ok(value) = serde_json::from_value::<GameEditionPaths>(path) {
                            paths.insert(name, value);
                        }
                    }

                    paths
                }

                None => HashMap::new()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameEditionPaths {
    pub game: Driver,
    pub addons: Driver
}

impl GameEditionPaths {
    pub fn default_for_edition(game: &integrations::Game, edition: &integrations::standards::game::Edition) -> anyhow::Result<Self> {
        Ok(Self {
            game: Driver::PhysicalFsDriver {
                base_folder: LAUNCHER_FOLDER
                    .join("games")
                    .join(&game.game_title)
                    .join(&edition.title)
                    .join("Game")
            },

            addons: Driver::PhysicalFsDriver {
                base_folder: LAUNCHER_FOLDER
                    .join("games")
                    .join(&game.game_title)
                    .join(&edition.title)
                    .join("Addons")
            }
        })
    }
}
