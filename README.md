# Cobalt

## Features
The following feature(s) are currently implemented:
* File addition at runtime
* Toggleable separate mods
* Extra settings
* Gameplay tweaks (Smithy on battle preparations, ...)
* XML merging (not all files are supported yet)
* MSBT merging
* Localization support

The following feature(s) are planned or being worked on:
* Expanding memory limits (also known as the dependency IPS patches)
* ...

Explanations and instructions to use the features can be found on the [Cobalt Wiki](https://github.com/Raytwo/Cobalt/wiki).

## Downloads 
Head to the [release](https://github.com/Raytwo/Cobalt/releases/latest) page to get the latest build.

## Installation and usage
Only version 2.0.0 of Fire Emblem: Engage is supported.

<details>
  <summary>Switch</summary>
  
  1. Make sure your Atmosphere CFW is up-to-date
  2. Extract files on your SD to ``/atmosphere/contents/0100a6301214e000/exefs/``, feel free to create the directories if they are missing
  3. Create a directory on your SD if it doesn't already exist: ``/engage/mods/``
  4. Boot game
</details>
<details>
  <summary>Ryujinx</summary>
  
  1. Right click on the game in your list, select "Open Mod directory"
  2. Extract the files in the ``/skyline/exefs/`` directory, create them if missing
  3. Right click on the game in your list, select "Open Atmosphere Mods directory"
  4. Navigate back to the directory called ``sdcard``
  5. Create a directory on your SD if it doesn't already exist: ``/engage/mods/``
  6. Boot game
</details>

Put your TOML files in ``/engage/mods/<directory name of your choice>/`` for them to be detected by Cobalt.

## Bug reports
In the case where you are certain the issue comes from Cobalt itself, consider [opening an issue on this repository](https://github.com/Raytwo/Cobalt/issues/new). Eventually provide a screenshot of the error message to make it easier.

## Where are the sources?
The code is not ready to be used by other people for the time being, as the project this runs on is unreleased for the same reasons. This might change in the future.
If you are the kind of person who only uses open-source homebrews, I'd rather you don't come to rant about it and just don't use this.

## Special thanks
In no particular order: ``Shadów``, ``blujay``, ``jam1garner``, ``Moonling``, ``Sierra``, ``DeathChaos``, ``Thane``, ``Sohn``
