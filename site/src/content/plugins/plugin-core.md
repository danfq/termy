---
title: plugin_core Reference
description: Manifest schema and RPC protocol types in termy_plugin_core
order: 3
category: Plugins
---

`termy_plugin_core` defines the canonical plugin manifest and wire protocol shared by host and SDKs.

## Constants

- `PLUGIN_MANIFEST_FILE_NAME = "termy-plugin.json"`
- `PLUGIN_PROTOCOL_VERSION = 1`

## Exported API (complete)

### Structs

```rust
PluginManifest
PluginContributions
PluginCommandContribution
HostHello
HostCommandInvocation
PluginHello
PluginLogMessage
PluginToastMessage
DiscoveredPlugin
```

### Enums

```rust
PluginRuntime
PluginPermission
HostRpcMessage
PluginRpcMessage
PluginCapability
PluginLogLevel
PluginToastLevel
PluginManifestError
```

### Methods

```rust
PluginManifest::from_json_str(contents: &str) -> Result<PluginManifest, PluginManifestError>
PluginManifest::validate(&self) -> Result<(), PluginManifestError>
DiscoveredPlugin::resolved_entrypoint(&self) -> PathBuf
```

### Error variants

```rust
PluginManifestError::Json
PluginManifestError::MissingField
PluginManifestError::UnsupportedSchemaVersion
```

## Manifest

Primary type: `PluginManifest`

Required fields:

- `schema_version` (must be `1`)
- `id`
- `name`
- `version`
- `entrypoint`

Optional fields include:

- `description`
- `author`
- `minimum_host_version`
- `api_version`
- `runtime` (`executable`)
- `autostart` (defaults to `true`)
- `permissions`
- `contributes.commands`

Validation entrypoints:

- `PluginManifest::from_json_str`
- `PluginManifest::validate`

## Permissions

`PluginPermission` values:

- `filesystem_read`
- `filesystem_write`
- `network`
- `shell`
- `clipboard`
- `notifications`
- `terminal_read`
- `terminal_write`
- `ui_panels`

## Host -> Plugin messages

`HostRpcMessage`:

- `hello` (`HostHello`)
- `invoke_command` (`HostCommandInvocation`)
- `shutdown`
- `ping`

## Plugin -> Host messages

`PluginRpcMessage`:

- `hello` (`PluginHello`)
- `log` (`PluginLogMessage`)
- `toast` (`PluginToastMessage`)
- `pong`

## Capabilities

`PluginCapability` values:

- `command_provider`
- `event_subscriber`
- `ui_panel`
