pub struct StatBlock {
    pub original_message: Option<String>,
    pub jsonified_message: Option<String>,
}

impl super::stat_puller::FromDiscordMessage for StatBlock {
    fn new() -> Self {
        return Self {
            original_message: None,
            jsonified_message: None,
        };
    }

    const PROMPT: &'static str = r#"
        You are a stat pulling program. 
        Following this prompt you will receive a block of stats.
        Use the following schema:
        {
            "name": (string),
            "level": (number),
            "hunger": (number),
   
            "actions": (number),
            "reactions": (number),
    
            "speed": (number),
            "armor": (number),
            "hp": (number),
            "current_hp": (number),
            "hpr": (number),

            "energy_pool": (number),            
    
            "hit_die_per_level": (number)d(number),
            "stat_die_per_level": (number)d(number),
            "spell_die_per_level": (number)d(number),
            "stat_points_saved": (number)d(number),
            "spell_points_saved": (number)d(number),

    
            "stats": {
                "str": (number),
                "agl": (number),
                "con": (number),
                "wis": (number),
                "int": (number),
                "cha": (number),
                "kno": (number),
            }
        }    
        If there are missing values, interpret them as null
        If you are expecting a value in a specific format but it is incorrect, instead set the value as 'ERROR - (explanation)'
        You should translate these stats into a json dictionary.
        All keys should be lower case and spell corrected. Respond with only valid json    
    "#;

    fn jsonified_message_mut(&mut self) -> &mut Option<String> {
        &mut self.jsonified_message
    }
    fn original_message_mut(&mut self) -> &mut Option<String> {
        &mut self.original_message
    }
}
