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

