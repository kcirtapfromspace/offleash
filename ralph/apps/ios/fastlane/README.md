fastlane documentation
----

# Installation

Make sure you have the latest version of the Xcode command line tools installed:

```sh
xcode-select --install
```

For _fastlane_ installation instructions, see [Installing _fastlane_](https://docs.fastlane.tools/#installing-fastlane)

# Available Actions

## iOS

### ios build_debug

```sh
[bundle exec] fastlane ios build_debug
```

Build the app for testing (Debug)

### ios beta

```sh
[bundle exec] fastlane ios beta
```

Push a new beta build to TestFlight

### ios beta_manual

```sh
[bundle exec] fastlane ios beta_manual
```

Push a new beta build to TestFlight (manual signing)

Options:

  groups: Comma-separated TestFlight group names to distribute to (e.g., 'Beta Testers,Internal')

  build_number: Custom build number (default: timestamp)

  changelog: What's new in this build

### ios sync_dev_certs

```sh
[bundle exec] fastlane ios sync_dev_certs
```

Sync development certificates

### ios sync_appstore_certs

```sh
[bundle exec] fastlane ios sync_appstore_certs
```

Sync App Store certificates

### ios create_certs

```sh
[bundle exec] fastlane ios create_certs
```

Create new certificates (run once for new projects)

### ios test

```sh
[bundle exec] fastlane ios test
```

Run unit tests

### ios ui_test

```sh
[bundle exec] fastlane ios ui_test
```

Run UI tests

### ios bump_version

```sh
[bundle exec] fastlane ios bump_version
```

Increment version number

### ios version_info

```sh
[bundle exec] fastlane ios version_info
```

Get current version and build number

----

This README.md is auto-generated and will be re-generated every time [_fastlane_](https://fastlane.tools) is run.

More information about _fastlane_ can be found on [fastlane.tools](https://fastlane.tools).

The documentation of _fastlane_ can be found on [docs.fastlane.tools](https://docs.fastlane.tools).
