# Ordis
Ordis is a discord bot I am creating in order to familiarize myself with rust

## Features

### Dice roller

``/roll [dice string]`` will roll a given number of a given dice type

Examples: 
``/roll dice:8d10, 5d5, d6, 2d100`` (commas are optional):
```markdown
Rolling...
    - 8 D10s: (51)      [6,6,8,5,6,9,6,5]
    - 5 D5s: (12)      [4,3,3,1,1]
    - A D6: (2)
    - 2 D100s: (177)      [88,89]

```

``/roll dice:d6``
```markdown
Rolling...
    - A D6: (3)
```


