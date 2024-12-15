# Cobalt

## Features

> [!NOTE]
> Explanations and instructions to use the features can be found on the [Cobalt Wiki](https://github.com/Raytwo/Cobalt/wiki).

The following feature(s) are currently implemented:
* File addition at runtime
* Toggleable separate mods
* Extra settings
* Gameplay tweaks (Smithy and Skill inheritance on battle preparations, ...)
* XML merging
* MSBT merging
* Localization support ([we are looking for contributors!](https://github.com/Raytwo/Cobalt/wiki/Localization))
* New script commands
* Gamedata and MSBT reloading at runtime
* Vibrations in combat
* Phoenix Mode
* Auto-update

The following feature(s) are planned or being worked on:
* Expanding memory limits (also known as the dependency IPS patches)
* ...

## Downloads  
``GitHub``: Head to the [release](https://github.com/Raytwo/Cobalt/releases/latest) page to get the latest build.  
``Nexus Mods``: Cobalt is also available on [Nexus Mods](https://www.nexusmods.com/fireemblemengage/mods/2), but be aware that releases might be slightly delayed there.

## Installation and usage
> [!WARNING]
> Only version 2.0.0 of Fire Emblem Engage is supported.

<details>
  <summary>Switch</summary>
  
  1. Make sure your Atmosphere CFW is up-to-date
  2. Extract ``release.zip`` at the root of your SD
  3. Create a directory on your SD if it doesn't already exist: ``/engage/mods/``
  4. Boot game
</details>
<details>
  <summary>Ryujinx</summary>
  
  1. Press ``File > Open Ryujinx folder`` in the menu bar at the top
  2. Navigate to the ``sdcard`` folder.
  3. Extract the ``release.zip`` archive here.
  4. Create the following directory if it doesn't already exist: ``/engage/mods/``
  5. Boot game
</details>

> [!CAUTION]
> Please be aware that Yuzu has received a DMCA takedown notice from Nintendo and as such will not be supported by Cobalt anymore.
> This also goes for Suyu, and any kind of off-brand lemonade inspired name slapped over the source code of Yuzu.  
> The instructions remain here for the time being, should you want to try using it anyways, but please do not send us bug reports if it doesn't work.  
> They will not be addressed.
> 
<details>
  <summary>Yuzu (mobile unsupported, support discontinued)</summary>

  
  
  1. Press ``File > Open yuzu folder`` in the menu bar at the top
  2. Navigate to the ``sdmc`` folder.
  3. Extract the ``release.zip`` archive here.
  4. Create the following directory if it doesn't already exist: ``/engage/mods/``
  5. Boot game

  Please note that the auto-update feature and logs are not available for the time being. No, we can not fix this.
</details>

Boot the game, make sure the Cobalt version is shown on the bottom right of the Title Scene, and you're good to go.  
Proceed to the Wiki to find out about the [various features](https://github.com/Raytwo/Cobalt/wiki) available!

## Bug reports
> [!IMPORTANT]
> Before opening an issue, make sure to consult the [Wiki](https://github.com/Raytwo/Cobalt/wiki) to see if your problem is already addressed.  
> Assuming your problem is already explained in the Wiki and you still opened an issue, we might close the issue without replying.
> 
> Issues are primarily meant for Cobalt contributors to keep track of genuine problems, so please do your due diligence.

In the case where you are certain the issue comes from Cobalt itself, consider [opening an issue on this repository](https://github.com/Raytwo/Cobalt/issues/new/choose). Eventually provide a screenshot of the error message to make it easier.

## How do I build this?
For now, you don't! Use it as reference for plugins.

## Special thanks
In no particular order: ``Shadów``, ``blujay``, ``jam1garner``, ``Moonling``, ``Sierra``, ``DeathChaos25``, ``Thane98``, ``Sohn``, ``DogeThis``, ``TildeHat``
