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
* Localization support ([we are looking for contributors!](https://github.com/Raytwo/Cobalt/wiki/Multiple-languages))
* New script commands
* Gamedata and MSBT reloading at runtime
* Vibrations in combat
* Phoenix Mode
* Auto-update
* Expanding memory limits (also known as the dependency IPS patches)
* ...

## Installation and usage
> [!WARNING]
> Only version 2.0.0 of Fire Emblem Engage is supported.

> [!TIP]
> After installing, check your current version of Cobalt in the bottom-right corner of the Title Screen.
> If visible, you can then check the [Wiki](https://github.com/Raytwo/Cobalt/wiki) to find out how to use mods!

### Installer
``Cobalt Installer``: Visit the [Cobalt Installer](https://github.com/DivineDragonFanClub/cobalt-installer) page to get an automatic installer for Ryujinx and/or Nintendo Switch.

### Manual
``GitHub``: Head to the [release](https://github.com/Raytwo/Cobalt/releases/latest) page to get the latest build.  
``Nexus Mods``: Cobalt is also available on [Nexus Mods](https://www.nexusmods.com/fireemblemengage/mods/2), but be aware that releases might be slightly delayed there.

<details>
  <summary>Switch</summary>
  
  1. Make sure your Atmosphere CFW is up-to-date
  2. Extract ``release.zip`` at the root of your SD
  3. Create a directory on your SD if it doesn't already exist: ``/engage/mods/``
  4. Boot game
</details>
<details>
  <summary>Ryujinx</summary>
  
  1. Press ``File > Open Ryujinx folder`` in the menu bar at the top. Do NOT right click the game and open the Mods folder.
  2. Navigate to the ``sdcard`` folder.
  3. Extract the ``release.zip`` archive here.
  4. Create the following directory if it doesn't already exist: ``/engage/mods/``. This has to be done in the ``sdcard`` directory from step 2, NOT the ``atmosphere`` or Ryujinx mod folder.
  5. Boot game
</details>
<details>
  <summary>Yuzu (mobile unsupported, support discontinued)</summary>

  
  
  1. Press ``File > Open yuzu folder`` in the menu bar at the top
  2. Navigate to the ``sdmc`` folder.
  3. Extract the ``release.zip`` archive here.
  4. Create the following directory if it doesn't already exist: ``/engage/mods/``
  5. Boot game

  Please note that the auto-update feature and logs are not available for the time being. No, we can not fix this.
</details>

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
In no particular order: ``Shadów``, ``blujay``, ``jam1garner``, ``Moonling``, ``Sierra``, ``DeathChaos25``, ``Thane98``, ``Sohn``, ``DogeThis``, ``TildeHat``, ``MistressAshai``
