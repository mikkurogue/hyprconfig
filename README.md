# Hyprconfig - The hyprland gui settings tool for idots

![build](https://github.com/mikkurogue/hyprconfig/actions/workflows/rust.yml/badge.svg)

Basically just a small utility to on the fly edit the hyprland coniguration settings that are defined.
Some people are just bad at reading the docs, and some are bad at understanding words (me), so I wanted 
to create a tool that will allow users to find, re-define or change their settings from a gui application.

I am a firm beliver that if Linux needs more wider adoption, and especially Hyprland as well, we should
have an ecosystem that is also "noob" or "normie" friendly.

## Overrides
This project (on first run) will create and append a new `conf-overrides.conf` to your hyprland config.
This will by itself also then write all overrides into this new file.

This file is meant to not be very organized or "readable" as its only job is to exist as a configuration file.
Also the only easy configurations that are settable for now are ones that provide us with unique identifiers until i can figure out a good solution.

I.e. monitors, as we can fetch monitors with `hyprctl monitors all` and get every connected monitor
For inputs like mouse (sensitivity) and keyboard (layout/locale) is easy enough to also do

## Status

Project is very much still in progress, I'm working on the basics and writing down what I still need to figure out.

Feedback and contributions are welcome :)
