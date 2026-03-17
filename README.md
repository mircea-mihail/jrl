# jrl

**jrl** is a terminal-based journaling app for taking notes, writing descriptions, rating your day, and answering prompts.

Everything is stored locally, so your data is as safe as your computer.

---

## Installation Guide

To install **jrl** locally you can run:

```bash
$ cargo install jrl
```
Or just copy the executable to /usr/local/bin.

Another common first step is to call the jrl executable in your config file so you get promted a new question every time you open a new terminal. This config file can be found in your home directory and a common name for it is ```.bashrc```. In order to only sometimes get asked questions you can use the -s (sometimes) flag.

Add the following in your .bashrc file:

```bash
jl -s
```

In order to replace the quesitons file used for prompts, create a questions.txt file in the current directory using the template provided in the repository and run:

```bash
$ jrl --install-questions
```

## Questions file

This file contains all the questions you want to be asked when calling jrl or when doing jrl -s in your bashrc. Each question has 2 flags that change how long the answer from the user should be:
  * s: for short questions
  * l: for long ones

The **short questions** are meant to be answered with numbers or short text, while the **long questions** expect a multi line answer. 

One example of a question file I once used is the one below, mostly tracking health habits I was interested in at the time. Place this file in the ~/.jrl directory manually or just use the command mentioned in the previous section to do it automatically.

```
s: Number of coffees? 
s: Grams of sugar?
s: Cups of tea?
s: Hours of sleep?
s: Ate meat?
s: Number of beers?

l: What's a nice thing from today?
l: What's an upsetting thing from today?
```

---

## Usage

```bash
jrl [OPTIONS]
```

---

## Options

| Flag | Long Option | Argument | Description |
|------|------------|----------|-------------|
| `-d` | `--description` | `[DESCRIPTION]` | Talk about how your day was |
| `-n` | `--note` | `[NOTE]` | Add a short note during the day |
| `-r` | `--rating` | `[RATING]` | Rate your day out of 10 (can be any number) |
| `-s` | `--sometimes` | `[SOMETIMES]` | Lower chances of a question being asked (`true` / `false`) |
| `-e` | `--entries` | `[<ENTRIES>]` | Show all entries into the journal [possible values: true, false]. True by default |
| `-u` | `--update` | `<UPDATE>` | Update journal from x days ago |
|      |  `--install-questions`| `[<INSTALL_QUESTIONS>]` | Use the questions from the questions.txt file in the current directory [possible values: true, false]. True by default |
| `-a` | `--analytics` | `[<ANALYTICS>]` | Show analytics for the past x journal entries. 30 by default |
| `-h` | `--help` | `-` | Print help |


---

## Examples

Most flags work either by providing the argument immediately after the flag, or by entering the flag alone and typing your input on the following lines.


```
$ jrl
Hours of sleep?
>> 9

$ jrl -d "Today was productive and calm"

$ jrl -d
Add a description about the day:
>> Loved how everything just felt right 
>> in the morning played some guitar. I feel like i'm starting to get back into that habbit
>> later in the evening met my friends at the bar and we had so much fun 
>> 

$ jrl -n "i'm at home and i feel so boared, just want to lay down and do nothing all day"

$ jrl -n
Add a note during the day:
>> just got the greatest idea: what if everyone just had wings and could fly everywhere
>> 

$ jrl -r 8

$ jrl -u 1 -d
Update entry for 18 Feb 2026:
Add a description about the day:
>> Today was pretty cool, did some uni work in the morning then stayed late with the boys at the pub
>>

$ jrl -a 7
Analytics for the past 7 entries:
...

$ jrl -a | less
Analytics for the past 30 entries:
...
```

## Interactive mode

You can enter interactive mode with the following command:

```bash
$ jrl -e
```

This is a pager like interface to browse through all of your journal entries. To move up, down and to the next or previous files you can use the arrow keys, or using the classic vi layout, with the keys ```h```, ```j```, ```k``` and ```l```. You can also move down by pressing ```Enter``` and down by pressing ```Backspace```. 

To go directly to the top of the file press ```g``` and to go to the bottom press ```G```. 

To exit interactive mode press ```q``` or ```Esc```. 
