# Ordis
Ordis is a discord bot I am creating in order to familiarize myself with rust

## Features

### Dice roller

``/roll [dice string]`` will roll a given number of a given dice type

Examples: 
``/roll dice:8d10, 5d5, d6, 2d100`` (commas are optional):
```markdown

**Rolling**...

- 8 D10s: (**43**)            [4,6,1,8,8,2,8,6]
- 5 D5s: (**8**)              [1,1,4,1,1]
- 1 D6: (**4**)
- 2 D100s: (**79**)           [75,4]
__                                           __
Total: 134

```

``/roll dice:d6``
```markdown
Rolling...
    - A D6: (3)
```

### OpenAI/GPT

``/ask [prompt]`` will ask the configured openai model [prompt] and return the result in chat (currently this is restricted to one user ID, configured in OPENAI_AUTHORIZED. This will at some point be changed to allow marking users as authorized in the database.
