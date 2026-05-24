# display-profile

A CLI tool to save and apply Windows display configurations.

A *profile* is the set of display configurations, which contains the resolution, frequency rate, scaling, rotation, etc. for each active display.


# Installation

From the GitHub repository:
```shell
cargo install --git https://github.com/erdavila/windows-display-profile.git --bin display-profile
```

From the local checked-out repository:
```shell
cargo install --path 'PATH-TO\windows-display-profile\cli'
```

# Usage

Saving the profile:
```shell
display-profile save PROFILE.json
```

Applying the profile:
```shell
display-profile apply PROFILE.json
```

Validating the profile:
```shell
display-profile validate PROFILE.json
```

## Without installing

Via Cargo:
```shell
cargo run ACTION PROFILE.json
```

Via [Just](https://just.systems/man/en/installation.html):
```shell
just profile ACTION PROFILE.json
```


# Cargo Feature

When the `dump` feature is active, each [Windows CCD API] function call also generates a `dump-*.json` file containing the inputs and outputs of the call.

When installing: include the `--features dump` argument.

When running via Cargo:
```shell
cargo run --features dump ACTION PROFILE.json
```

When running via Just:
```shell
just profile-dump ACTION PROFILE.json
```


[Windows CCD API]: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/connecting-and-configuring-displays
