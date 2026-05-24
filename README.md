# `windows-display-profile`

Provides a CLI tool and Rust libs related to Windows display configurations:
* [`display-profile`](./cli/) - a CLI tool to save and apply Windows display configurations.
* [`display-profile-lib`](./lib/) - a lib with functions and types to get and set Windows display configurations.
* [`windows-ccd`](./windows-ccd/) - a lib with some convenience utilities to use the [Windows CCD API].


## Development
Dependency: [`just`](https://just.systems/man/en/installation.html)

```bash
just --list
```


[Windows CCD API]: https://learn.microsoft.com/en-us/windows-hardware/drivers/display/connecting-and-configuring-displays
