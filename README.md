# Markdown Diary Parser (MDP)

MDP is a small CLI tool to help keeping the overview in large Markdown diary files. A
Markdown diary is defined here as a Markdown file which contains dates (in YYYY-MM-DD
format) as its H1 headers. MDP allows to list and search for tags, e.g. "@holidays",
in the diary. Furthermore it can also show all tasks written down in the diary (e.g.
"TODO: Clean Room" or "DONE: Clean Kitchen").

## Features
- List all tags in a Markdown diary file
- Search for tags in a Markdown diary file using multiple search terms (which can be combined with AND/OR)
- List all tasks in a Markdown diary file
- Show tree of Markdown tokens in a Markdown diary file

## Example Markdown file compatible with MDP

```markdown
# 2022-11-02

## School

@school

Today in school something happened.

## Freetime

After school I went home

DONE: Clean room

---

# 2022-11-03

## Meeting

In the morning i had a meeting with @roger (roger.example@gmail.com).

TODO: Inform roger about the state of the project
```

## Usage

### Show help

```
$ mdp --help 
Usage: mdp <COMMAND>

Commands:
search      Search for tags
tags        List tags
token-tree  Show tree of Markdown content/tokens
tasks       Show all tasks (TODO, TODO UNTIL <DATE>, DOING, REVIEW, DONE)
help        Print this message or the help of the given subcommand(s)

Options:
-h, --help     Print help
-V, --version  Print version
```

### List tags

``` 
$ mdp tags data/mdp_example_file1.md
Tag                       Count
roger                         1
school                        1
```

### Search for tags

```
$ mdp search data/mdp_example_file1.md roger
# 2022-11-03

## Meeting

In the morning i had a meeting with @roger (roger.example@gmail.com).
```

### List tasks

```
$ mdp tasks mdp_example_file1.md
TODO: Inform roger about the state of the project
```

### Show token tree

```
$ mdp token-tree mdp_example_file1.md

├─ # 2022-11-02
│  ├─ ## School
│  │  ├─ @school
│  │  └─ Today in school something happened.
│  └─ ## Freetime
│     ├─ After school I went home
│     └─ DONE: Clean room
└─ # 2022-11-03
   └─ ## Meeting
      ├─ In the morning i had a meeting with
      ├─ @roger
      ├─  (
      ├─ roger.example@gmail.com
      ├─ ).
      └─ TODO: Inform roger about the state of the project
```

Also have a look at the options of all the commands!

## Contributing
You have a question about the package or you would like to have a certain feature implemented? Open an issue!

## Authors

* **Mathias Aschwanden** 

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
