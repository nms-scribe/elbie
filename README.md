# Elbie

Elbie is a rust library to be used for generating words in a fictional language. This isn't just a random word generator. It uses rules based on the way natural human languages work. You define the phonemes and the patterns used to generate words, and it does the work. It will also let you generate spelling systems for them.

It also includes a tool for transforming the phonemic words from said languages into something else. This can be used for applying regular phonological changes. It could also be used for creating a more complex orthography by treating graphemes as phonemes.

Also included is a simple tool called "goblin", which can be used as an example of how to build your own tools to work with your own languages. There is also a transformation to "hobgoblin", which is a dialect of the language. The goblin language is based on some words used by the named people in a certain popular role-playing game. Unfortunately, I've lost that list of words via failing to check them into version control. I suspect they were copyrighted anyway.

Elbie is a work in progress which I created for my own personal use. I'm releasing it into the wild because I feel that others might have use for it too. If you find it to be of any use, let me know.

## Usage

Elbie is a rust library, although it contains functions that make it very easy to write your own command-line tool. See the goblin sub-project for an example of how this is done. It is not published to crates.io and I have no intentions to do so, unless I get a lot of interest and maybe some funding for maintenance.

If you wish to use it, create a rust project and add the following to the dependencies in your `Cargo.toml`:

```
[dependencies]
elbie = { git = "https://github.com/nms-scribe/elbie.git" }
```

If you have a specific revision/commit hash of the crate that you would prefer: 

```
[dependencies]
elbie = { git = "https://github.com/nms-scribe/elbie.git", rev = "5d729af" }
```
