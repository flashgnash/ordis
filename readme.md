![image](https://github.com/user-attachments/assets/66794d5b-328b-4d97-b181-cdc234eaad1c)# Ordis
Ordis is a discord bot I am creating in order to familiarize myself with rust, and facilitate playing Mir

## Features

### Mir

#### Dice roller

`/roll [dice string]` will roll a given number of a given dice type

`/roll` with no arguments will roll 1d100 - default roll to be configurable in a future update

##### Examples:

`/roll dice:8d10, 5d5, d6, 2d100` (commas are optional):

![image](https://github.com/user-attachments/assets/913325ac-e774-48a3-921a-92040bc96bac)

If a character sheet has been setup, you can roll with modifiers from your character's stat block

`/roll dice:1d100+str`

![image](https://github.com/user-attachments/assets/fc479b6a-4b2d-4ecd-a028-3d80681f8c7f)

#### Character sheet

##### Important to note for hosting the bot (skip if someone else has already set it up):

An openAI token is required for parsing character sheets and spell sheets
(I have chosen to use GPT for parsing character sheets as they are hand written and so typos/capitalisation/formatting inconsistencies are possible, which chatgpt can correct for)

##### Importing character sheet and spell list

You can import a character sheet into Ordis by clicking the Create Character button from the context menu on the character sheet's message in discord

![image](https://github.com/user-attachments/assets/1cee918d-f3a7-4cfe-994f-0789db13dea2)

It expects a section of a character sheet in the following format:

(in the case of stats, as long as they are under the header any stat names can be used and will be usable in the roll command)

(there should probably also be a way to genericise the level up dice, as they have recently been changed from Stat Die, Hit Die and Spell Die Per Lev)

```markdown
Name : Hank
Level : 29

Actions : 2
Reactions : 2
Speed : 90
Armour :

HP : 138
Current Hp : 80
HPR :

Energy Die Per Lev : 1d4
Magic Die Per Lev : 1d7
Training Die Per Lev : 1d8

Stats:
Str - 10
Agl - 30
Con - 15
Wis - 25
Int - 20
Cha - 20
```

It is also recommended to set your spell list message for the ability to use the spell casting system, this is done the same way as the character sheet, except with Set Spell Message instead
(it will set the spell list for your currently selected character so make sure to select it with /list_characters and /select_character first if you have multiple)

![image](https://github.com/user-attachments/assets/ee1ab4ab-58a9-42ac-b822-cc2e4619258f)

This is at the moment very specific to Mir's spell system, a reasonable amount of modification would be required to apply this same system to D&D
(not to mention the fact that D&D does not have mana in the first place, though it could be adapted to track spell slots instead)

##### Character Commands

`/list_characters` will list all your current registered characters and their IDs (this is currently the only way to get character IDs)
`/delete_character (character ID)` will delete and deselect your character by the ID you provide
`/select_character (character ID)` will set your default character to the ID provided

#### Level up command

Once your stat block is setup, you can run /level_up (number of levels), and the bot will automatically roll the appropriate number of die for the amount of levels you have gained, sum them all and output them in chat

`/level_up 3` results in
![image](https://github.com/user-attachments/assets/c83e5120-ca0d-4d67-8897-21e9ee71c437)

#### Status command

The /status command will provide a summary of your current health, mana, hunger, and active toggle spells
![image](https://github.com/user-attachments/assets/27cc95ae-51e8-48da-92ef-353076aef16f)

#### Mana & Spell system

The mana system consists of a few commands

##### Spell system

The spell system is a layer built ontop of the mana system in order to simplify deducting mana for frequently cast spells and toggle spells

- `/list_spells` will list all the spells the bot knows about on your currently selected character
- `/cast_spell (name)` will attempt to deduct the mana from your mana pool for the spell of that name, and spit out an error message if you don't hvae enough to cast it (if the spell is a toggle spell, it will toggle it on and not take mana, more on that later)
- `/end_turn` will deduct the mana cost of all active toggle spells in the order they were enabled in, and fail if you run out of mana (it will attempt to deduct the mana of all active spells regardless of if the previous one failed so be careful)

##### Mana system

You can also manipulate mana directly with the following:

`/get_mana` - Prints out your current mana bar in an ephemeral message, similar to `/status`
`/set_mana (value)` - sets your current mana to the given value
`/add_mana (value)` - Increases your mana by the given value (this can be negative to decrease mana instead)
`/sub_mana (value)` - Decreases your mana by the given value (this can be negative to increase mana instead)
`/mod_mana (expression)` - Modifies your mana by the given mathematical expression where 'n' represents your current mana (e.g `/mod_mana n*2` doubles your current mana, `/mod_mana n+(5*2)` increases your mana by 10, etc

### OpenAI/GPT

`/ask [prompt]` will ask the configured openai model [prompt] and return the result in chat (currently this is restricted to one user ID, configured in OPENAI_AUTHORIZED. This will at some point be changed to allow marking users as authorized in the database.

`/draw [prompt]` will generate an image based on your prompt using Dall-E, sharing the same token environment variable
