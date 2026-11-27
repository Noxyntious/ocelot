# Ocelot
A Minecraft pre-launch mod downloader.

Downloads mods from Modrinth (planned: CurseForge) and automatically installs them into your mod directory. Leaves mods not in the pack alone.

## Format
The file format for Ocelot's modpacks (.ocpack) is as follows:

```
mod_url,version,platform,required
```

For example:

```
https://modrinth.com/mod/fabric-api,26.3,fabric,required
```

A complete modpack file consumed by Ocelot would look like this:
```
mod_url,version,platform,required
https://modrinth.com/mod/fabric-api,26.3,fabric,required
https://modrinth.com/mod/sodium,26.3,fabric,required
https://modrinth.com/mod/xaeros-minimap,26.3,fabric,optional
https://modrinth.com/mod/iris,26.3,fabric,optional
```

This is essentially just CSV, and as such can be created in any CSV editor, like Excel.

## Usage

Ocelot is used from the command line, designed for the "Pre-launch Command" field in Prism Launcher.

```
ocelot-linux-x86_64 https://my-website.com/my-pack.ocpack $INST_MC_DIR/mods
```

PrismLauncher then resolves `$INST_MC_DIR` into the absolute path of the .minecraft folder in the given instance.

```
ocelot-linux-x86_64 https://my-website.com/my-pack.ocpack /home/me/.local/share/PrismLauncher/instances/bestmodpackever/minecraft/mods
```

You can run ocelot outside PrismLauncher, although why you'd want to do that is unknown.