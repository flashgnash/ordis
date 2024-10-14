use std::fmt;

pub struct SpellSheet {
    pub original_message: Option<String>,
    pub jsonified_message: Option<String>,
}

impl fmt::Display for SpellSheet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(json) = self.jsonified_message.as_ref() {
            return write!(f, "{json}");
        }
        if let Some(message) = self.original_message.as_ref() {
            return write!(f, "{message}");
        }

        write!(f, "No spell sheet found")
    }
}
impl super::stat_puller::FromDiscordMessage for SpellSheet {
    fn new() -> Self {
        return Self {
            original_message: None,
            jsonified_message: None,
        };
    }

    const PROMPT: &'static str = r#"
    You are a spell list pulling program. 
    Following this prompt you will receive a block of spells and their costs.
    Use the following schema:    
    {

        "spells": {
            fireball": {
                "type": "single",
                "cost": -150,
                "cast_time": "1 turn"
            },
            "invisibility": {
                "type": "toggle"
                "cost": -50,
                "cast_time": "instant"
            },
            "regen": {
                "type": "toggle",
                "cost": 50,
                "cast_time": "1 turn"
            }
        }
    }    

    If there are missing values, interpret them as null
    For cast time, use the middle value that should look like '2 actions'
    If there are spaces in spell names, remove them, replacing them with underscores
    If you are expecting a value in a specific format but it is incorrect, instead set the value as 'ERROR - (explanation)'
    You should translate these spells into a json dictionary.
    All keys should be lower case and spell corrected. Respond with only valid json"                    
"#;

    fn jsonified_message_mut(&mut self) -> &mut Option<String> {
        &mut self.jsonified_message
    }
    fn original_message_mut(&mut self) -> &mut Option<String> {
        &mut self.original_message
    }
}
